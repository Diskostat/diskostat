use std::{
    fmt,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, RwLock,
    },
    thread,
};

use anyhow::{Context, Result};
use jwalk::{
    DirEntry,
    Parallelism::{RayonNewPool, Serial},
    WalkDirGeneric,
};

use crate::ui::event_handling::DiskoEvent;

use super::{
    entry_node::{EntryNode, EntryNodeView},
    entry_size::EntrySize,
    tree_walk_state::{CustomJWalkClientState, TreeWalkAncestor, TreeWalkState},
};

use ref_tree::{Node, Tree};

pub enum BackpropOperation {
    Add,
    Subtract,
}

#[derive(Default)]
pub struct DiskoTree {
    tree: Arc<RwLock<Tree<EntryNode>>>,
    current_directory: Option<Arc<RwLock<Node<EntryNode>>>>,
    traversal_handler: Option<thread::JoinHandle<()>>,
    root: PathBuf,
    traversal_threads: usize,
    is_traversing: Arc<AtomicBool>,
    stop_traversing: Arc<AtomicBool>,
}

// Public interface

impl DiskoTree {
    pub(crate) fn new(root: PathBuf, traversal_threads: usize) -> Self {
        Self {
            tree: Arc::new(RwLock::new(Tree::new())),
            current_directory: None,
            traversal_handler: None,
            root,
            traversal_threads,
            is_traversing: Arc::new(AtomicBool::new(false)),
            stop_traversing: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn root_path(&self) -> PathBuf {
        self.root.clone()
    }

    fn get_children(
        node: &std::sync::RwLockReadGuard<'_, Node<EntryNode>>,
        sort_by_disk_size: bool,
    ) -> Vec<EntryNodeView> {
        let mut children: Vec<EntryNodeView> = node
            .get_children()
            .iter()
            .enumerate()
            .map(|(index, child)| {
                let child = child
                    .read()
                    .expect("Failed to read child while getting children");
                let mut entry = EntryNodeView::from_entry_node(&child.data);
                entry.index_to_original_node = Some(index);
                entry
            })
            .collect();

        children.sort_by(|a, b| {
            if sort_by_disk_size {
                b.sizes.disk_size.cmp(&a.sizes.disk_size)
            } else {
                b.sizes.apparent_size.cmp(&a.sizes.apparent_size)
            }
        });

        children
    }

    /// Switch the current working directory to its parent.
    /// Returns an error if the current directory is not set, i.e., the
    /// traversal has not yet computed a root or if the current directory
    /// has no parent.
    pub(crate) fn switch_to_parent_directory(&mut self) -> Result<()> {
        let current_directory_arc = self
            .current_directory
            .clone()
            .context("Current directory not set")?;
        let current_directory = current_directory_arc
            .read()
            .expect("Failed to read current directory");
        let parent = current_directory
            .get_parent()
            .context("Failed to get parent of current directory")?
            .upgrade()
            .expect("Failed to upgrade weak reference to parent of current directory");
        self.current_directory = Some(parent);
        Ok(())
    }

    /// Switch the current working directory to its child at the given index.
    /// Returns an error if the current directory is not set, i.e., the
    /// traversal has not yet computed a root or if the index is out of bounds.
    pub(crate) fn switch_to_subdirectory(&mut self, index: usize) -> Result<()> {
        let current_directory_arc = self
            .current_directory
            .take()
            .context("Current directory not set")?;
        let current_directory = current_directory_arc
            .read()
            .expect("Failed to read current directory");
        let subdir_arc = current_directory
            .get_child_at(index)
            .context("Failed to get child at given index")?;
        self.current_directory = Some(subdir_arc);
        Ok(())
    }

    /// Get the view of the current directory and its children.
    /// Returns `None` if the current directory is not set, i.e., the traversal
    /// has not yet computed a root.
    pub(crate) fn get_current_dir_view(
        &mut self,
        sort_by_disk_size: bool,
    ) -> Option<(EntryNodeView, Vec<EntryNodeView>)> {
        if self.current_directory.is_none() {
            self.current_directory = self
                .tree
                .read()
                .expect("Failed to read the underlying tree in diskotree")
                .get_root();
        }
        let current_directory = self
            .current_directory
            .as_ref()?
            .read()
            .expect("Failed to read current directory");
        let children = Self::get_children(&current_directory, sort_by_disk_size);
        let current_directory_view = EntryNodeView::from_entry_node(&current_directory.data);
        Some((current_directory_view, children))
    }

    /// Get the view of the subdirectory of the current directory at the given
    /// index.
    /// Returns `None` if the current directory is not set, i.e., the traversal
    /// has not yet computed a root, or if the index is out of bounds.
    pub(crate) fn get_subdir_of_current_dir_view(
        &self,
        index: usize,
        sort_by_disk_size: bool,
    ) -> Option<Vec<EntryNodeView>> {
        let subdir_arc = {
            let current_directory = self
                .current_directory
                .as_ref()?
                .read()
                .expect("Failed to read current directory");
            current_directory.get_child_at(index)?
        };
        let subdir = subdir_arc
            .read()
            .expect("Failed to read subdir while getting subdir view");

        Some(Self::get_children(&subdir, sort_by_disk_size))
    }

    fn jwalk_walk_dir(
        root: PathBuf,
        tree: Arc<RwLock<Tree<EntryNode>>>,
        traversal_threads: usize,
    ) -> WalkDirGeneric<(TreeWalkState, ())> {
        WalkDirGeneric::<(TreeWalkState, ())>::new(root)
            .sort(true)
            .parallelism(if traversal_threads == 1 {
                Serial
            } else {
                RayonNewPool(traversal_threads)
            })
            .skip_hidden(false)
            .root_read_dir_state(TreeWalkState::new(tree))
            .process_read_dir(|depth, dir_path, state, children| {
                Self::process_dir(depth, dir_path, state, children);
            })
    }

    /// Starts the traversal on a separate thread. Once the computation
    /// ends, the file system from given root path is evauluated and sizes
    /// are calculated.
    /// This method is non-blocking.
    pub(crate) fn start_background_traversal(&mut self, sender: mpsc::Sender<DiskoEvent>) {
        let tree = self.tree.clone();
        let is_traversing = self.is_traversing.clone();
        let stop_traversing = self.stop_traversing.clone();
        let root = self.root.clone();
        let traversal_threads = self.traversal_threads;
        self.traversal_handler = Some(thread::spawn(move || {
            is_traversing.store(true, Ordering::Release);

            for _ in Self::jwalk_walk_dir(root, tree, traversal_threads) {
                if stop_traversing.load(Ordering::Relaxed) {
                    break;
                }
            }

            is_traversing.store(false, Ordering::Release);
            // Here we just ignore if the event handler has stopped.
            let _ = sender.send(DiskoEvent::TraversalFinished);
        }));
    }

    /// Check if the traversal thread is still running.
    pub(crate) fn is_traversing(&self) -> bool {
        self.is_traversing.load(Ordering::Acquire)
    }

    /// Stops the background traversal.
    /// Blocks the calling thread until the traversal thread completely stops.
    pub(crate) fn stop_background_traversal(&mut self) {
        self.stop_traversing.store(true, Ordering::Relaxed);
        if let Some(handler) = self.traversal_handler.take() {
            handler.join().expect("Failed to join traversal thread.");
        }
    }

    pub(crate) fn traverse(&mut self) {
        for _ in Self::jwalk_walk_dir(self.root.clone(), self.tree.clone(), self.traversal_threads)
        {
        }
    }

    pub(crate) fn delete_entries(&self, mut indices: Vec<usize>) -> Result<()> {
        let current_directory_arc = self
            .current_directory
            .clone()
            .context("Current directory not set")?;

        let children = {
            let current_directory = current_directory_arc
                .read()
                .expect("Failed to write current directory");

            current_directory.get_children()
        };

        let mut deleted_size = EntrySize::default();

        // When deleting children, we modify the size of the vector containing children,
        // hence indices may no longer correspond to desired children. Therefore we sort them in descending order,
        // so children are deleted from end and hence we omit the neccessity to adjust indices upon child deletion.
        indices.sort_unstable();
        indices.reverse();

        for child_index in indices {
            let child = children
                .get(child_index)
                .context("Provided index is out of bounds.")?;
            let child_data = {
                let child = child.clone();
                let read_child = child
                    .read()
                    .expect("Failed to read child while deleting children.");

                read_child.data.clone()
            };

            child_data.delete_entry()?;

            deleted_size += child_data.sizes;
            self.tree
                .clone()
                .write()
                .expect("Failed to write to tree while deleting children.")
                .remove_subtree(child)
                .expect("Failed to delete child.");
        }

        Self::backprop_size(
            &current_directory_arc,
            deleted_size,
            BackpropOperation::Subtract,
        );
        Ok(())
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
        let Some((dir_node, dir_size)) = EntryNode::new_dir(dir_path) else {
            return;
        };

        // Count size of file children.
        let mut size = dir_size;

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
                    child_node.sizes = EntrySize::default();
                }
                size += child_node.sizes;
                Tree::attach_child(&node, child_node);
            });

        // Propagate size up including this node to root (including).
        Self::backprop_size(&node, size, BackpropOperation::Add);

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

    fn backprop_size(
        node: &Arc<RwLock<Node<EntryNode>>>,
        size: EntrySize,
        operation: BackpropOperation,
    ) {
        let iter = Tree::iter_to_root_from_node(node.clone());

        iter.into_iter().for_each(|node| {
            let mut node = node
                .write()
                .expect("Failed to write while backpropagating size");

            match operation {
                BackpropOperation::Add => node.data.sizes += size,
                BackpropOperation::Subtract => node.data.sizes -= size,
            };
        });
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
