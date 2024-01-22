use std::sync::{Arc, RwLock};

use super::node::Node;

pub struct NodeToRootIterator<T> {
    node: Arc<RwLock<Node<T>>>,
}

impl<T> NodeToRootIterator<T> {
    pub fn new(node: Arc<RwLock<Node<T>>>) -> Self {
        Self { node }
    }
}

impl<T> Iterator for NodeToRootIterator<T> {
    type Item = Arc<RwLock<Node<T>>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Borrow current node and read parent.
        let parent_opt = {
            let current = self.node.read().unwrap();
            current.parent.clone()
        };

        // If parent is None, we are at the root node.
        let Some(parent) = parent_opt else {
            return None;
        };

        // Step up.
        match parent.upgrade() {
            Some(parent) => {
                self.node = parent.clone();
                Some(parent)
            }
            None => None,
        }
    }
}
