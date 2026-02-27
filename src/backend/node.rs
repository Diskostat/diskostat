use std::{
    fmt::Debug,
    sync::{Arc, RwLock, Weak},
};

use diskostat::RwLockExt;

#[derive(Debug)]
pub struct Node<T> {
    pub data: T,
    children: Vec<Arc<RwLock<Node<T>>>>,
    parent: Weak<RwLock<Node<T>>>,
}

impl<T> Node<T> {
    pub fn children(&self) -> &[Arc<RwLock<Node<T>>>] {
        &self.children
    }

    pub fn parent(&self) -> Option<Arc<RwLock<Node<T>>>> {
        self.parent.upgrade()
    }

    /// Returns the child at the given index, or `None` if the index is out of bounds.
    pub fn get_child(&self, index: usize) -> Option<Arc<RwLock<Node<T>>>> {
        self.children.get(index).cloned()
    }

    /// Removes the child at the given index.
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    pub fn remove_child(&mut self, index: usize) {
        self.children.remove(index);
    }
}

impl<T> Node<T> {
    pub(crate) fn new(data: T) -> Self {
        Self {
            children: vec![],
            data,
            parent: Weak::new(),
        }
    }

    pub fn new_shared(data: T) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            data,
            children: vec![],
            parent: Weak::new(),
        }))
    }

    /// Creates a new node with `data` and attaches it to `node` as a child.
    ///
    /// Returns the newly created child.
    pub fn create_and_attach_child(node: &Arc<RwLock<Node<T>>>, data: T) -> Arc<RwLock<Node<T>>> {
        let mut child_node = Node::new(data);
        child_node.parent = Arc::downgrade(node);

        let child = Arc::new(RwLock::new(child_node));
        node.write_unwrap().children.push(child.clone());

        child
    }
}

impl<T: Default> Default for Node<T> {
    fn default() -> Self {
        Self {
            data: T::default(),
            children: Vec::default(),
            parent: Weak::default(),
        }
    }
}

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
        let current = self.node.clone()?;

        // Step up the tree.
        self.node = current.read_unwrap().parent();

        // Return the current node.
        Some(current)
    }
}

#[cfg(test)]
#[test]
fn node_new() {
    let node = Node::new(0);
    assert_eq!(node.data, 0);
    assert!(node.children.is_empty());
    assert!(node.parent().is_none());
}

#[test]
fn create_and_attach_child() {
    let root = Node::new_shared(0);
    let child_1 = Node::create_and_attach_child(&root, 1);
    let child_2 = Node::create_and_attach_child(&root, 2);

    assert_eq!(root.read().unwrap().children.len(), 2);
    assert!(root.read().unwrap().parent.upgrade().is_none());

    assert!(root
        .read()
        .unwrap()
        .children
        .iter()
        .any(|c| Arc::ptr_eq(c, &child_1)));

    assert!(root
        .read()
        .unwrap()
        .children
        .iter()
        .any(|c| Arc::ptr_eq(c, &child_2)));

    assert!(Arc::ptr_eq(
        &child_1.read().unwrap().parent.upgrade().unwrap(),
        &root
    ));
    assert!(Arc::ptr_eq(
        &child_2.read().unwrap().parent.upgrade().unwrap(),
        &root
    ));

    assert_eq!(child_1.read().unwrap().data, 1);
    assert_eq!(child_2.read().unwrap().data, 2);
    assert!(child_1.read().unwrap().children.is_empty());
    assert!(child_2.read().unwrap().children.is_empty());
}

#[test]
fn node_to_root_iterator() {
    let root = Node::new_shared(1);
    let child = Node::create_and_attach_child(&root, 10);
    let _ = Node::create_and_attach_child(&root, 20);
    let nested = Node::create_and_attach_child(&child, 100);
    let _ = Node::create_and_attach_child(&nested, 1000);

    let mut iter = NodeToRootIterator::new(root.clone());
    assert_eq!(iter.next().unwrap().read().unwrap().data, 1);
    assert!(iter.next().is_none());

    let mut iter = NodeToRootIterator::new(nested);
    assert_eq!(iter.next().unwrap().read().unwrap().data, 100);
    assert_eq!(iter.next().unwrap().read().unwrap().data, 10);
    assert_eq!(iter.next().unwrap().read().unwrap().data, 1);
    assert!(iter.next().is_none());
}
