use std::{sync::Arc, fmt::{Debug, Formatter}};

use slab_tree::{NodeRef, Tree, TreeBuilder};

use super::entry_node::EntryNode;



#[derive(Clone)]
pub(crate) struct TreeWalkState {
    pub(crate) parent: Option<Arc<NodeRef<'static, EntryNode>>>,
    pub(crate) tree: Arc<Tree<EntryNode>>,
}

impl Default for TreeWalkState {
    fn default() -> Self {
        Self { parent: None, tree: Arc::new(TreeBuilder::new().build()) }
    }
}

impl Debug for TreeWalkState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeWalkState")
            .finish()
    }
}

pub(crate) type CustomJWalkClientState = (TreeWalkState, ());
