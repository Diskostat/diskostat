use std::{
    fmt,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use jwalk::{DirEntry, Parallelism::RayonNewPool, WalkDirGeneric};

use super::model::{
    entry_node::EntryNode,
    tree_walk_state::{CustomJWalkClientState, TreeWalkAncestor, TreeWalkState},
};

use ref_tree::{Node, Tree};

pub struct DiskoTree {
    tree: Arc<RwLock<Tree<EntryNode>>>,
    root_path: PathBuf,
}

// Public interface

impl DiskoTree {
    pub(crate) fn new(starting_at: PathBuf) -> Self {
        Self {
            tree: Arc::new(RwLock::new(Tree::new())),
            root_path: starting_at,
        }
    }

    /// Poll the state of evaluation by taking look (.read()) at the
    /// tree building.
    pub(crate) fn get_tree(&self) -> Arc<RwLock<Tree<EntryNode>>> {
        self.tree.clone()
    }

    /// Run this on separate thread to not block the ui thread. Once
    /// the computation ends, the file system from given root path is
    /// evaluated and sizes calcuated.
    pub(crate) fn traverse(&mut self) {
        let walk_dir = WalkDirGeneric::<(TreeWalkState, ())>::new(self.root_path.clone())
            .sort(true)
            .parallelism(RayonNewPool(10))
            .skip_hidden(false)
            .root_read_dir_state(TreeWalkState::new(self.tree.clone()))
            .process_read_dir(|depth, dir_path, state, children| {
                Self::process_dir(depth, dir_path, state, children);
            });

        let mut iter = walk_dir.into_iter();

        while iter.next().is_some() {}
    }
}

// Convenience/helpers

impl DiskoTree {
    fn process_dir(
        depth: Option<usize>,
        dir_path: &Path,
        state: &mut TreeWalkState,
        children: &mut [jwalk::Result<DirEntry<CustomJWalkClientState>>],
    ) {
        // Skip parent directory (./..).
        if depth.is_none() {
            return;
        }
        // Create entry node from jwalks
        let Some(dir_node) = EntryNode::new_dir(dir_path) else {
            return;
        };

        let mut size = dir_node.metadata.len();

        // Create node on tree.
        let node = Self::attach_to_tree(state, dir_node);

        children
            .iter_mut()
            // Put reference to results inner types.
            .map(|dir_entry_result| dir_entry_result.as_ref())
            // Filter errors out and return just `DirEntry` entries.
            .filter_map(std::result::Result::ok)
            .filter(|dir_entry| dir_entry.file_type.is_file())
            // Map to our `EntryNode`s.
            .map(EntryNode::try_from)
            // Throw away when convertion failed.
            .filter_map(Result::ok)
            // Finaly process the file children.
            .for_each(|mut child_node| {
                if state.file_has_been_seen(&child_node.metadata) {
                    child_node.size = 0;
                }
                size += child_node.size;
                Tree::attach_child(&node, child_node);
            });

        // Propagate size up including this node to root (including).
        Self::write_size_upwards_including(&node, size);

        // Move (i.e. not .clone()) reference to this node as a parent
        // for the next iteration.
        state.ancestor = TreeWalkAncestor::Parent(node);
    }

    fn attach_to_tree(state: &TreeWalkState, node: EntryNode) -> Arc<RwLock<Node<EntryNode>>> {
        match &state.ancestor {
            TreeWalkAncestor::Parent(parent) => Tree::attach_child(parent, node),
            TreeWalkAncestor::Tree(tree) => tree
                .write()
                .expect("Writing to tree failed when setting root.")
                .create_node_and_set_root(node)
                .expect(
                    "The inner tree already has a root node but disko tree thinks it does not yet.",
                ),
        }
    }

    fn write_size_upwards_including(node: &Arc<RwLock<Node<EntryNode>>>, size: u64) {
        let iter = Tree::iter_to_root_from_node(node.clone());
        for node in iter {
            node.write()
                .expect("Failed to write while propagating size up")
                .data
                .size += size;
        }
    }
}

impl fmt::Display for DiskoTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tree = self.tree.read().expect("Failed to read tree");
        let Some(root) = tree.get_root() else {
            return write!(f, "Empty DiskoTree");
        };
        let root = root
            .read()
            .expect("Failed to read root while printing disko tree");

        write!(f, "{}", root.data)?;
        let children = root.get_children();
        std::mem::drop(root); // Drop the lock on root.

        let Some((last, rest)) = children.split_last() else {
            return Ok(());
        };

        for child in rest {
            let child = child
                .read()
                .expect("Failed to read child while printing disko tree");
            write!(f, "\n├── {}", child.data)?;
        }
        let last = last
            .read()
            .expect("Failed to read last child while printing disko tree");
        write!(f, "\n└── {}", last.data)
    }
}
