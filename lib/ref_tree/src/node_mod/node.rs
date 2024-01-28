use std::{
    fmt::Debug,
    sync::{Arc, RwLock, Weak},
};

#[derive(Debug)]
pub struct Node<T> {
    /// empty vec -> leaf node (only in tree struct, not in fs,
    /// i.e. can be empty dir)
    pub(crate) children: Vec<Arc<RwLock<Node<T>>>>,

    pub data: T,

    /// None -> root node
    /// Weak -> prevent reference cycles
    pub(crate) parent: Option<Weak<RwLock<Node<T>>>>,
}

// Public interface

impl<T> Node<T> {
    pub fn get_children(&self) -> Vec<Arc<RwLock<Node<T>>>> {
        self.children.clone()
    }

    pub fn get_parent(&self) -> Option<Weak<RwLock<Node<T>>>> {
        self.parent.clone()
    }

    /// Returns the child at the given index.
    /// Returns `None` if index is out of bounds.
    pub fn get_child_at(&self, index: usize) -> Option<Arc<RwLock<Node<T>>>> {
        self.children.get(index).cloned()
    }
}

// Internal convenience functions

impl<T> Node<T> {
    /// Creates disconnected Node.
    pub(crate) fn new(data: T) -> Self {
        Self {
            children: vec![],
            data,
            parent: None,
        }
    }

    /// Attaches given node to this node as a child.
    /// Does NOT attach parent.
    ///
    /// Consumes given Node. Returns newly created Arc-RwlLock for the new node.
    pub(crate) fn create_and_attach_child(&mut self, data: T) -> Arc<RwLock<Node<T>>> {
        let other_node_arc = Arc::new(RwLock::new(Node::new(data)));
        self.children.push(other_node_arc.clone());

        other_node_arc
    }

    pub(crate) fn attach_parent(&mut self, parent: &Arc<RwLock<Node<T>>>) {
        self.parent = Some(Arc::downgrade(parent));
    }
}
