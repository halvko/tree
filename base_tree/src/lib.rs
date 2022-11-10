use std::ptr::{self, NonNull};

pub struct Node<T> {
    data: T,

    // These three ptr::NonNull _always_ has to be dereferentiable, and must not be accessible from
    // outside the structure (aka, be created from mutable references)
    left: Option<ptr::NonNull<Node<T>>>,
    right: Option<ptr::NonNull<Node<T>>>,
    parent: Option<ptr::NonNull<Node<T>>>,
}

// This implementation keeps the invariant that a mutable reference to a node, means exclusive access to its children and parent, if present.
impl<T> Node<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            parent: None,
            left: None,
            right: None,
        }
    }

    pub fn replace_right<'a>(
        &'a mut self,
        new_child: Option<&'a mut Self>,
    ) -> Option<&'a mut Self> {
        let self_ref = self.into();
        let child = &mut self.right;
        unsafe { Self::replace_child_helper(self_ref, child, new_child) }
    }

    pub fn replace_left<'a>(&'a mut self, new_child: Option<&'a mut Self>) -> Option<&'a mut Self> {
        let self_ref = self.into();
        let child = &mut self.left;
        unsafe { Self::replace_child_helper(self_ref, child, new_child) }
    }

    /// # Safety
    ///
    /// `old_child_ref`
    unsafe fn replace_child_helper<'a>(
        parent: NonNull<Self>,
        old_child_ref: &mut Option<ptr::NonNull<Self>>,
        new_child: Option<&'a mut Self>,
    ) -> Option<&'a mut Self> {
        // Clear parent
        let old_child = old_child_ref.take().map(|mut ptr| {
            // Safety: Our invariant ensures us exclusive access to ptr
            let ptr = unsafe { ptr.as_mut() };
            ptr.parent = None;
            ptr
        });

        // Miri does not like this for some reason
        let new_child = new_child.map(|nc| {
            nc.parent = Some(parent);
            nc
        });

        *old_child_ref = new_child.map(|ptr| ptr.into());

        old_child
    }

    pub fn get(&self) -> &T {
        &self.data
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }

    pub fn left(&self) -> Option<&Self> {
        self.left.map(|ptr| unsafe { ptr.as_ref() })
    }

    pub fn right(&self) -> Option<&Self> {
        self.right.map(|ptr| unsafe { ptr.as_ref() })
    }

    pub fn parent(&self) -> Option<&Self> {
        self.parent.map(|ptr| unsafe { ptr.as_ref() })
    }

    pub fn left_mut(&mut self) -> Option<&mut Self> {
        self.left.map(|mut ptr| unsafe { ptr.as_mut() })
    }

    pub fn right_mut(&mut self) -> Option<&mut Self> {
        self.right.map(|mut ptr| unsafe { ptr.as_mut() })
    }

    pub fn parent_mut(&mut self) -> Option<&mut Self> {
        self.parent.map(|mut ptr| unsafe { ptr.as_mut() })
    }

    pub fn split_mut(&mut self) -> (Option<&mut Self>, &mut Self, Option<&mut Self>) {
        // Safety: We previously had exclusive access to the whole tree, now we remove all
        // references between self and its children, giving exclusive access to each of the
        // subtrees.
        let remove_parent = |mut ptr: ptr::NonNull<Self>| {
            let ptr = unsafe { ptr.as_mut() };
            ptr.parent = None;
            ptr
        };

        let left = self.left.take().map(remove_parent);
        let right = self.right.take().map(remove_parent);

        (left, self, right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walk_around() {
        let node0 = &mut Node::new(String::from("0"));
        let node1 = &mut Node::new(String::from("1"));
        let node2 = &mut Node::new(String::from("2"));
        let node3 = &mut Node::new(String::from("3"));
        let node4 = &mut Node::new(String::from("4"));

        //      2
        //    /   \
        //   0     3
        //  / \   / \
        // -   1 -   4
        node0.replace_right(Some(node1));
        node3.replace_right(Some(node4));
        node2.replace_left(Some(node0));
        node2.replace_right(Some(node3));

        let Some(node0) = node2.left_mut() else {
            panic!("Expected node 0 to be present")
        };
        assert_eq!(node0.get(), "0");
        *node0.get_mut() = "10".into();
        let None = node0.left() else {
            panic!("Expected nothing to the left of node 0")
        };
        let Some(node1) = node0.right_mut() else {
            panic!("Expected node 1 to be present")
        };
        assert_eq!(node1.get(), "1");
        let Some(node10) = node1.parent_mut() else {
            panic!("Expected node 1 to have a parent")
        };
        assert_eq!(node10.get(), "10");
        let Some(n4) = node10.parent_mut().and_then(Node::right_mut).and_then(Node::right_mut) else {
            panic!("Expected to be able to walk to node 4")
        };
        assert_eq!(n4.get(), "4");
    }
}
