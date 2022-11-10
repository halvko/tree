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

        // Set parent
        let new_child = new_child.map(|nc| {
            nc.parent = Some(parent.into());
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

    pub fn left<'a>(&'a self) -> Option<&'a Self> {
        self.left.map(|ptr| unsafe { ptr.as_ref() })
    }

    pub fn right<'a>(&'a self) -> Option<&'a Self> {
        self.right.map(|ptr| unsafe { ptr.as_ref() })
    }

    pub fn parent<'a>(&'a self) -> Option<&'a Self> {
        self.parent.map(|ptr| unsafe { ptr.as_ref() })
    }

    pub fn left_mut<'a>(&'a mut self) -> Option<&'a mut Self> {
        self.left.map(|mut ptr| unsafe { ptr.as_mut() })
    }

    pub fn right_mut<'a>(&'a mut self) -> Option<&'a mut Self> {
        self.right.map(|mut ptr| unsafe { ptr.as_mut() })
    }

    pub fn parent_mut<'a>(&'a mut self) -> Option<&'a mut Self> {
        self.parent.map(|mut ptr| unsafe { ptr.as_mut() })
    }

    pub fn split_mut<'a>(
        &'a mut self,
    ) -> (Option<&'a mut Self>, &'a mut Self, Option<&'a mut Self>) {
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
    fn creation() {
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

        let node0 = node2.left_mut();
        assert_eq!(node0.map(|n| &n.data), Some(&String::from("0")));
    }
}
