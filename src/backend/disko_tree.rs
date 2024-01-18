use std::{
    error::Error,
    fmt::{Debug, Formatter},
    fs,
    path::{Path, PathBuf},
    sync::Arc, ops::Deref,
};

use crate::backend::model::{self};
use jwalk::{DirEntry, Parallelism::RayonNewPool, WalkDirGeneric};

use super::model::*;
use super::{
    model::{
        entry_node::EntryNode,
        tree_walk_state::{CustomJWalkClientState, TreeWalkState},
    },
    types::*,
};
use slab_tree::*;

pub struct DiskoTree {
    tree: Arc<Tree<EntryNode>>,
}

type DirEntryResult = &'static mut Result<DirEntry<(TreeWalkState, ())>, dyn Error>;

impl DiskoTree {
    pub fn new() -> Self {
        Self {
            tree: Arc::new(TreeBuilder::new().build()),
        }
    }

    pub fn traverse(&'static self) {
        let walk_dir = WalkDirGeneric::<(TreeWalkState, ())>::new(".")
            .sort(true)
            .parallelism(RayonNewPool(10))
            .process_read_dir(|depth, dir_path, state, children| {
                self.process_dir(depth, dir_path, state, children)
            });

        let mut iter = walk_dir.into_iter();

        while iter.next().is_some() {}
    }

    fn process_dir(
        &self,
        depth: Option<usize>,
        dir_path: &Path,
        state: &mut TreeWalkState,
        children: &mut Vec<jwalk::Result<DirEntry<CustomJWalkClientState>>>,
    ) {
        let Some(dir_node) = EntryNode::new_dir(dir_path) else {
            return;
        };
        // TODO: create slabtree node for this dir

        // 1. Count size of children which are files

        let mut size = 0;

        children
            .iter_mut()
            .filter_map(|dir_entry_result|
                        dir_entry_result.as_ref().ok()
            )
            .filter(|dir_entry| dir_entry.file_type.is_file())
            .filter_map(|dir_entry|
                 EntryNode::new(dir_entry.clone())
            )
            .for_each(|entry_node| {
                size += entry_node.size;

                // Self::attach(entry_node, parent)

        });

        println!("{:12} | {}", size, dir_node.name);

        // 2. Create node for myself with the size calcualeted
        // create firstly to attach then files - leaf nodes

        // 3. check if paretnt if so -> append myself to it else -> set myself as root

        // 4. state.parent = myself // for downstream recursion
    }


    fn attach(entry_node: EntryNode, parent: NodeRef<EntryNode>) {

    }
}
