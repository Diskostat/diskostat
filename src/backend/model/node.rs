use std::{sync::{Arc, RwLock, Weak}, fmt::{Display, Debug}};

use crate::backend::model::tree::Tree;

use super::entry_node::EntryNode;


#[derive(Debug)]
pub struct Node<T> {
    /// empty vec -> leaf node (only in tree struct, not in fs,
    /// i.e. can be empty dir)
    pub(super) children: Vec<Arc<RwLock<Node<T>>>>,

    pub(crate) data: T,

    /// None -> root node
    /// Weak -> prevent reference cycles
    pub(super) parent: Option<Weak<RwLock<Node<T>>>>
}


impl<T> Display for Node<T> where T: Display + Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "Node with data: {}\n\
                - parent: {:?}\n\
                - children: {:?}\n",
               self.data, self.parent, self.children)
    }
}

impl<T> Node<T> {
    /// Creates not connected Node
    pub(crate) fn new(data: T) -> Self {
        Self { children: vec![], data, parent: None }
    }

    /// Attaches given node to this node as a child.
    /// Does NOT attach parant.
    ///
    /// Consumes given Node. Returns newly created Arc-RwlLock for the new node.
    pub(crate) fn attach_child(&mut self, node: Node<T>) -> Arc<RwLock<Node<T>>> {
        let other_node_arc = Arc::new(RwLock::new(node));
        self.children.push(other_node_arc.clone());

        other_node_arc
    }

    // Overrides parent to the given node.
    // pub(crate) fn attach_parent(&mut self, parent: Arc<RwLock<Node<T>>>) {
    //     self.parent = Some(Arc::downgrade(&parent));
    // }

    pub(crate) fn get_children(&self) -> Vec<Arc<RwLock<Node<T>>>> {
        self.children.clone()
    }
}


impl Node<EntryNode> {
    // TODO: Create trait
    // TODO: User formatter as Display does
    pub(crate) fn pretty(&self) -> String {
        format!("{:<20} • {}", self.data.name, self.data.size)
    }
}

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
        let parent_opt =  {
            let current = self.node.read().unwrap();
            current.parent.clone()
        };

        // If parent is None, we are at the root node.
        let Some(parent) = parent_opt else { return None };

        // Stup up.
        match parent.upgrade() {
            Some(parent) => {
                self.node = parent.clone();
                Some(parent)
            },
            None => None
        }
    }
}



#[test]
fn test_node_new() {
    let node = Node::new(0);
    assert_eq!(node.data, 0);
    assert!(node.children.is_empty());
    assert!(node.parent.is_none());
}

#[test]
fn test_node_attach_child() {
    let mut node = Node::new(0);
    let child_node = Node::new(1);
    let child = node.attach_child(child_node);
    assert_eq!(node.children.len(), 1);
    assert_eq!(node.children[0].read().unwrap().data, 1);
    assert_eq!(child.read().unwrap().data, 1);
}

#[test]
fn test_node_get_children() {
    let mut node = Node::new(0);
    let child = Node::new(1);
    let child_arc = node.attach_child(child);
    let children = node.get_children();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].read().unwrap().data, 1);
    assert_eq!(child_arc.read().unwrap().data, 1);
}

#[test]
fn test_node_child_writability() {
    let mut node = Node::new(0);
    let child_node = Node::new(1);
    let child = node.attach_child(child_node);
    {
        let mut child = child.write().unwrap();
        child.data = 2;
    } // Drop write lock.
    assert_eq!(child.read().unwrap().data, 2);
    assert_eq!(node.get_children().first().unwrap().read().unwrap().data, 2);
}


#[test]
fn test_node_to_tree_iterator() {
    let mut tree = Tree::new();
    let root_node = tree.create_and_set_root(0).unwrap();
    let child_node = Node::new(1);
    let child = Tree::attach_child(root_node.clone(), child_node);
    let child2_node = Node::new(2);
    let child2 = Tree::attach_child(child.clone(), child2_node);

    let mut iter = NodeToRootIterator::new(child2);

    assert_eq!(iter.next().expect("next is none").read().unwrap().data, 1);
    assert_eq!(iter.next().unwrap().read().unwrap().data, 0);
    assert!(iter.next().is_none());
}
