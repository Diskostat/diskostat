use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use byte_unit::Byte;
use jwalk::{DirEntry, Parallelism::RayonNewPool, WalkDirGeneric};

use super::model::{
    entry_node::EntryNode,
    tree_walk_state::{CustomJWalkClientState, TreeWalkState},
};

use ref_tree::{Node, Tree};

pub enum BackpropOperation {
    Add,
    Subtract,
}

pub struct DiskoTree {
    tree: Arc<RwLock<Tree<EntryNode>>>,
}

impl DiskoTree {
    fn new() -> Self {
        Self {
            tree: Arc::new(RwLock::new(Tree::new())),
        }
    }

    pub(crate) fn new_static() -> &'static DiskoTree {
        Box::leak(Box::new(DiskoTree::new()))
    }

    pub(crate) fn traverse(&'static self) {
        let walk_dir = WalkDirGeneric::<(TreeWalkState, ())>::new(".")
            .sort(true)
            .parallelism(RayonNewPool(10))
            .root_read_dir_state(TreeWalkState::Tree(self.tree.clone()))
            .process_read_dir(|depth, dir_path, state, children| {
                Self::process_dir(depth, dir_path, state, children);
            });

        let mut iter = walk_dir.into_iter();

        while iter.next().is_some() {}
    }

    fn process_dir(
        _depth: Option<usize>,
        dir_path: &Path,
        state: &mut TreeWalkState,
        children: &mut [jwalk::Result<DirEntry<CustomJWalkClientState>>],
    ) {
        // Create entry node from jwalks
        let Some(dir_node) = EntryNode::new_dir(dir_path) else {
            return;
        };
        // Create not connected node to put into the tree then
        let mut dir_node = Node::new(dir_node);

        // count size + attach children
        let size = dir_node.data.size;

        // println!("started reading dir: {}", dir_node.name);

        children
            .iter_mut()
            .filter_map(|dir_entry_result| dir_entry_result.as_ref().ok())
            .filter(|dir_entry| dir_entry.file_type.is_file())
            .filter_map(EntryNode::new)
            .map(Node::new)
            .for_each(|node| {
                size.add(node.data.size);
                dir_node.attach_child(node);
            });

        dir_node.data.size = size;

        let node = Self::attach_to_tree(state, dir_node);

        // Propagate size to root.
        Self::backprop_size(&node, size, BackpropOperation::Add);

        // Move (i.e. not .clone()) reference to this node as a parent
        // for the next iteration.
        *state = TreeWalkState::Parent(node);
    }

    fn attach_to_tree(
        state: &TreeWalkState,
        node: Node<EntryNode>,
    ) -> Arc<RwLock<Node<EntryNode>>> {
        match state {
            TreeWalkState::Parent(parent) => Tree::attach_child(parent, node),
            TreeWalkState::Tree(tree) => tree.write().unwrap().set_root_node(node),
        }
    }

    fn backprop_size(
        node: &Arc<RwLock<Node<EntryNode>>>,
        size: Byte,
        operation: BackpropOperation,
    ) {
        let iter = Tree::iter_to_root_from_node(node.clone());

        iter.into_iter().for_each(|node| {
            let node = node
                .write()
                .expect("Failed to write while backpropagating size");

            match operation {
                BackpropOperation::Add => node.data.size.add(size),
                BackpropOperation::Subtract => node.data.size.subtract(size),
            };
        });
    }

    pub(crate) fn get_tree(&self) -> Arc<RwLock<Tree<EntryNode>>> {
        self.tree.clone()
    }

    pub(crate) fn delete_children(
        parent: Arc<RwLock<Node<EntryNode>>>,
        indexes: Vec<usize>,
    ) -> std::io::Result<()> {
        let read_parent = parent.clone().write().unwrap() else {
            panic!("Failed to read parent while deleting children");
        };

        let mut deleted_size: Byte = Byte::from_u64(0);

        indexes.into_iter().for_each(|index| {
            let Some(child) = read_parent.get_children().get(index);

            delete_entry(child)?.expect("Failed to delete entry ${}", child.path);
            deleted_size += child.size();
            read_parent.children().remove(index);
        });

        Self::backprop_size(&parent, deleted_size, BackpropOperation::Subtract);
        Ok(())
    }
}
