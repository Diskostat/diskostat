use std::sync::{Arc, RwLock};

use super::node::Node;

pub struct NodeToRootIterator<T> {
    node: Option<Arc<RwLock<Node<T>>>>,
}

impl<T> NodeToRootIterator<T> {
    pub fn new(node: Arc<RwLock<Node<T>>>) -> Self {
        Self { node: Some(node) }
    }
}

impl<T> Iterator for NodeToRootIterator<T> {
    type Item = Arc<RwLock<Node<T>>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Finish if we're at the root.
        let Some(current) = self.node.clone() else {
            return None;
        };

        // Step up the tree.
        self.node = current
            .read()
            .expect("Could not read current node.")
            .parent
            .clone()
            .and_then(|parent| parent.upgrade());

        // Return the current node.
        Some(current)
    }
}
