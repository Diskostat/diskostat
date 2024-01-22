use std::{sync::{Arc, Mutex, RwLock}, fmt::{Debug, Formatter}};

// use slab_tree::{NodeMut, Tree, TreeBuilder};

use super::entry_node::EntryNode;
use ref_tree::{Node, Tree};



// #[derive(Clone)]
// pub(crate) struct TreeWalkState {
    // pub(crate) parent: Option<Arc<NodeRef<'static, EntryNode>>>,
    // pub(crate) tree: Arc<Tree<EntryNode>>,
// }

#[derive(Clone)]
pub(crate) enum TreeWalkState {
    // Tree(Arc<Box<Tree<EntryNode>>>),
    // Parent(Arc<Mutex<Box<NodeMut<'static, EntryNode>>>>)
    Tree(Arc<RwLock<Tree<EntryNode>>>),
    Parent(Arc<RwLock<Node<EntryNode>>>)
}


impl Default for TreeWalkState {
    fn default() -> Self {
        // Self::Tree(Arc::new(Box::new(TreeBuilder::new().build())))
        // Self { parent: None, tree: Arc::new(TreeBuilder::new().build()) }
        Self::Tree(Arc::new(RwLock::new(Tree::new())))
    }
}

impl Debug for TreeWalkState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeWalkState")
            .finish()
    }
}

pub(crate) type CustomJWalkClientState = (TreeWalkState, ());
