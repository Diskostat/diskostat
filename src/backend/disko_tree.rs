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
        let mut node = Node::new(dir_node);



        // Put node into the tree

    }

    // fn process_dir(
    //     &self,
    //     depth: Option<usize>,
    //     dir_path: &Path,
    //     state: &mut TreeWalkState,
    //     children: &mut Vec<jwalk::Result<DirEntry<CustomJWalkClientState>>>,
    // ) {


    //     let myself = match state {
    //         TreeWalkState::Parent(parent) => {
    //             let mut mutex = parent.clone();
    //             let Ok(mut locked_parent) = mutex.lock() else { return };
    //             let node = locked_parent.append(dir_node.clone());
    //             node.node_id()
    //         },
    //         TreeWalkState::Tree(tree) => {
    //             let root_id = tree.clone().set_root(dir_node);


    //             // let mut mutex = tree.clone();
    //             // let Ok(mut locked_tree) = mutex.lock() else { return };
    //             // let root_id = locked_tree.set_root(dir_node.clone());
    //             // let Some(root_node) = locked_tree.get_mut(root_id) else { return };
    //             // root_node
    //             root_id
    //         }
    //     };

    //     let tree = self.tree.clone();
    //     let pointer_to_node = { ||

    //         // let Ok(tree) = tree.lock() else { return };


    //         1
    //     };




    //     // lock here
    //     // node = self.tree.(...).get(node_id);

    //     // if let ;
    //     // }

    //     // if let ;
    //     //         // let root_node = locked_tree.g
    //     //         // let mut tree = locked_tree.borrow_mut();



    //     // }

    //     // let myself = match state {
    //     //     TreeWalkState::Parent(parent) => {
    //     //         let arc = parent.clone();
    //     //         let Ok(mut locked_parent) = arc.lock() else { return };
    //     //         let mut parent = locked_parent.borrow_mut();
    //     //         let mut node = parent.append(dir_node);

    //     //         Box::new(node)

    //     //         // let root_id = tree.set_root(dir_node);
    //     //         // let Some(mut root_node) = tree.get_mut(root_id);
    //     //         // let temp = parent.clone();
    //     //         // let Ok(locked_parent) = temp.lock() else { return };
    //     //         // locked_parent.borrow_mut().append(dir_node)
    //     //         // Arc::new(Mutex::new(node))
    //     //     },
    //     //     TreeWalkState::Tree(tree) => {
    //     //         let mut mutex = tree.clone();
    //     //         let Ok(mut locked_tree) = mutex.lock() else { return };
    //     //         let mut tree = locked_tree.borrow_mut();

    //     //         let root_id = tree.set_root(dir_node);
    //     //         let Some(mut root_node) = tree.get_mut(root_id) else { return };

    //     //         // let Ok(mut locked_tree) = tree.clone().lock() else { return };
    //     //         // let mut tree_mut = locked_tree.borrow_mut();

    //     //         // let root_id = tree_mut.set_root(dir_node);
    //     //         // let Some(root_node) = tree_mut.get_mut(root_id) else { return };
    //     //         // root_node
    //     //         Box::new(root_node)
    //     //         // Arc::new(Mutex::new(root_node))
    //     //     }
    //     // };


    //     // state = parent(myself)

    //     // Filter



    //     // let node = slab_tree::new_node();



    //     // node creation write


    //     // if let Some(parent) =  state.parent.clone() {
    //     //     let x = parent.borrow_mut();
    //     // } else {


    //     // }


    //     // let this_dir_tree_node = Node

    //     // 1. Count size of children which are files

    //     let mut size = 0;

    //     // println!("started reading dir: {}", dir_node.name);

    //     children
    //         .iter_mut()
    //         .filter_map(|dir_entry_result|
    //                     dir_entry_result.as_ref().ok()
    //         )
    //         .filter(|dir_entry| dir_entry.file_type.is_file())
    //         .filter_map(|dir_entry|
    //              EntryNode::new(dir_entry.clone())
    //         )
    //         .for_each(|entry_node| {
    //             println!("reading size from: {}", entry_node.name);
    //             size += entry_node.size;

    //             // Self::attach(entry_node, parent)

    //     });

    //     // println!("{:12} | {}", size, dir_node.name);


    //     // println!("ended reading dir: {}", dir_node.name);
    //     // size computed

    //     // sencond write
    //     // add it to the node

    //     // 2. Create node for myself with the size calcualeted
    //     // create firstly to attach then files - leaf nodes

    //     // 3. check if paretnt if so -> append myself to it else -> set myself as root

    //     // 4. state.parent = myself // for downstream recursion
    // }


    // fn attach(entry_node: EntryNode, parent: NodeRef<EntryNode>) {

    // }
}
