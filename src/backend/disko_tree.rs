use std::{
    error::Error,
    fmt::{Debug, Formatter},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock}, ops::Deref, borrow::BorrowMut,
};

use
    crate::backend::model::{self};
use jwalk::{DirEntry, Parallelism::RayonNewPool, WalkDirGeneric};



use super::model::{*, tree::Tree, node::Node};
use super::{
    model::{
        entry_node::EntryNode,
        tree_walk_state::{CustomJWalkClientState, TreeWalkState},
    },
    types::*,
};
// use slab_tree::*;

pub struct DiskoTree {
    tree: Arc<RwLock<Tree<EntryNode>>>,
}

type DirEntryResult = &'static mut Result<DirEntry<(TreeWalkState, ())>, dyn Error>;

impl DiskoTree {
    pub fn new() -> Self {
        Self {
            // tree: Arc::new(Box::new(TreeBuilder::new().build())),
            // tree: Arc::new(Box::new(TreeBuilder::new().build())),
            tree: Arc::new(RwLock::new(Tree::new())),
        }
    }

    pub fn traverse(&'static self) {
        let walk_dir = WalkDirGeneric::<(TreeWalkState, ())>::new(".")
            .sort(true)
            .parallelism(RayonNewPool(10))
            .root_read_dir_state( TreeWalkState::Tree(self.tree.clone()) )
            .process_read_dir(|depth, dir_path, state, children| {
                self.process_dir(depth, dir_path, state, children)
            });

        let mut iter = walk_dir.into_iter();

        while let Some(Ok(item)) = iter.next() {
            // println!("{:?}", item.file_name);

        }
    }

    fn process_dir(
        &self,
        depth: Option<usize>,
        dir_path: &Path,
        state: &mut TreeWalkState,
        children: &mut Vec<jwalk::Result<DirEntry<CustomJWalkClientState>>>,
    ) {
        // Create entry node from jwalks
        let Some(dir_node) = EntryNode::new_dir(dir_path) else { return; };
        // Create not connected node to put into the tree then
        let mut dir_node = Node::new(dir_node);

        // count size + attach children
        let mut size = dir_node.data.size;

        // println!("started reading dir: {}", dir_node.name);

        children
            .iter_mut()
            .filter_map(|dir_entry_result|
                        dir_entry_result.as_ref().ok()
            )
            .filter(|dir_entry| dir_entry.file_type.is_file())
            .filter_map(|dir_entry|
                 EntryNode::new(dir_entry.clone())
            )
            .map(Node::new)
            .for_each(|node| {
                // println!("reading size from: {}", node.data.name);
                size += node.data.size;

                dir_node.attach_child(node);
        });

        // println!("{:12} | {}", size, dir_node.name);

        dir_node.data.size = size;

        let node = Self::attach_to_tree(state, dir_node);

        // Propagate size to root.
        Self::propagate_size_up(node.clone(), size);

        // Move (i.e. not .clone()) reference to this node as a parent
        // for the next iteration.
        *state = TreeWalkState::Parent(node);
    }

    fn attach_to_tree(state: &TreeWalkState, node: Node<EntryNode>) -> Arc<RwLock<Node<EntryNode>>> {
        match state {
            TreeWalkState::Parent(parent) => {
                Tree::attach_child(parent.clone(), node)
            },
            TreeWalkState::Tree(tree) => {
                tree.write().unwrap().set_root_node(node)
            }
        }
    }

    fn propagate_size_up(node: Arc<RwLock<Node<EntryNode>>>, size: u64) {
        let iter = Tree::iter_to_root_from_node(node.clone());
        for node in iter {
            node.write().expect("Failed to write while propagating size up").data.size += size;
        }
    }

    pub(crate) fn get_tree(&self) -> Arc<RwLock<Tree<EntryNode>>> {
        self.tree.clone()
    }
}