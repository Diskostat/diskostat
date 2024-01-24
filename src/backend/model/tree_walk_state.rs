use std::{
    fmt::{Debug, Formatter},
    sync::{Arc, RwLock},
};

use super::entry_node::EntryNode;
use ref_tree::{Node, Tree};

#[derive(Clone)]
pub(crate) enum TreeWalkState {
    Tree(Arc<RwLock<Tree<EntryNode>>>),
    Parent(Arc<RwLock<Node<EntryNode>>>),
}

impl Default for TreeWalkState {
    fn default() -> Self {
        Self::Tree(Arc::new(RwLock::new(Tree::new())))
    }
}

impl Debug for TreeWalkState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeWalkState").finish()
    }
}

pub(crate) type CustomJWalkClientState = (TreeWalkState, ());
