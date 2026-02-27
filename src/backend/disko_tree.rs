use std::{
    fmt, fs, mem,
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
    tree_walk_state::{CustomJWalkClientState, TreeWalkState},
};

use super::node::{Node, NodeToRootIterator};
use diskostat::RwLockExt;

pub struct DiskoTree {
    root: Arc<RwLock<Node<EntryNode>>>,
    current_directory: Arc<RwLock<Node<EntryNode>>>,
    traversal_handler: Option<thread::JoinHandle<()>>,
    path: PathBuf,
    traversal_threads: usize,
    is_traversing: Arc<AtomicBool>,
    stop_traversing: Arc<AtomicBool>,
}

// Public interface

impl DiskoTree {
    pub(crate) fn new(root: EntryNode, path: PathBuf, traversal_threads: usize) -> Self {
        let root = Node::new_shared(root);

        Self {
            root: root.clone(),
            current_directory: root,
            traversal_handler: None,
            path,
            traversal_threads,
            is_traversing: Arc::new(AtomicBool::new(false)),
            stop_traversing: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn root_path(&self) -> PathBuf {
        self.path.clone()
    }

    /// Get the views of the children of the given `node`.
    fn children_views(node: &Node<EntryNode>, sort_by_disk_size: bool) -> Vec<EntryNodeView> {
        let mut children: Vec<EntryNodeView> = node
            .children()
            .iter()
            .enumerate()
            .map(|(index, child)| {
                let child = child.read_unwrap();
                let mut entry = EntryNodeView::from(&child.data);

                entry.index_to_original_node = Some(index);
                entry
            })
            .collect();

        children.sort_by(|a, b| {
            if sort_by_disk_size {
                b.size.disk.cmp(&a.size.disk)
            } else {
                b.size.apparent.cmp(&a.size.apparent)
            }
        });

        children
    }

    /// Switch the current working directory to its parent.
    ///
    /// Returns an error if the current directory has no parent.
    pub(crate) fn switch_to_parent_directory(&mut self) -> Result<()> {
        let parent = self
            .current_directory
            .read_unwrap()
            .parent()
            .context("current directory has no parent")?;
        self.current_directory = parent;
        Ok(())
    }

    /// Switch the current working directory to its child at the given index.
    ///
    /// Returns an error if the index is out of bounds.
    pub(crate) fn switch_to_subdirectory(&mut self, index: usize) -> Result<()> {
        let subdir = self
            .current_directory
            .read_unwrap()
            .get_child(index)
            .context("Failed to get child at given index")?;
        self.current_directory = subdir;
        Ok(())
    }

    /// Get the view of the given `directory` and its children.
    pub(crate) fn view_of_directory(
        directory: &Node<EntryNode>,
        sort_by_disk_size: bool,
    ) -> (EntryNodeView, Vec<EntryNodeView>) {
        let children = Self::children_views(directory, sort_by_disk_size);
        let directory_view = EntryNodeView::from(&directory.data);
        (directory_view, children)
    }

    /// Get the view of the current directory and its children.
    pub(crate) fn current_directory_view(
        &mut self,
        sort_by_disk_size: bool,
    ) -> (EntryNodeView, Vec<EntryNodeView>) {
        let current_directory = self.current_directory.read_unwrap();

        Self::view_of_directory(&current_directory, sort_by_disk_size)
    }

    /// Get the view of the subdirectory of the current directory at the given
    /// index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub(crate) fn subdir_of_current_directory_view(
        &self,
        index: usize,
        sort_by_disk_size: bool,
    ) -> Option<Vec<EntryNodeView>> {
        let locked = self.current_directory.read_unwrap().get_child(index)?;
        let subdir = locked.read_unwrap();

        Some(Self::children_views(&subdir, sort_by_disk_size))
    }

    fn jwalk_walk_dir(
        root: Arc<RwLock<Node<EntryNode>>>,
        path: impl AsRef<Path>,
        traversal_threads: usize,
    ) -> WalkDirGeneric<(TreeWalkState, ())> {
        WalkDirGeneric::<(TreeWalkState, ())>::new(path)
            .sort(true)
            .parallelism(if traversal_threads == 1 {
                Serial
            } else {
                RayonNewPool(traversal_threads)
            })
            .skip_hidden(false)
            .root_read_dir_state(TreeWalkState::new(root))
            .process_read_dir(|depth, dir_path, state, children| {
                Self::process_dir(depth, dir_path, state, children);
            })
    }

    /// Starts the traversal on a separate thread. Once the computation
    /// ends, the file system from given root path is evauluated and sizes
    /// are calculated.
    /// This method is non-blocking.
    pub(crate) fn start_background_traversal(&mut self, sender: mpsc::Sender<DiskoEvent>) {
        let root = self.root.clone();
        let is_traversing = self.is_traversing.clone();
        let stop_traversing = self.stop_traversing.clone();
        let path = self.path.clone();
        let traversal_threads = self.traversal_threads;
        self.traversal_handler = Some(thread::spawn(move || {
            is_traversing.store(true, Ordering::Release);

            for _ in Self::jwalk_walk_dir(root, path, traversal_threads) {
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
        for _ in Self::jwalk_walk_dir(self.root.clone(), &self.path, self.traversal_threads) {}
    }

    pub(crate) fn delete_entries(&self, mut indices: Vec<usize>) -> Result<()> {
        let mut deleted_size = EntrySize::default();

        // When deleting children, we modify the size of the vector containing children,
        // hence indices may no longer correspond to desired children. Therefore we sort them in descending order,
        // so children are deleted from end and hence we omit the neccessity to adjust indices upon child deletion.
        indices.sort_unstable();
        indices.reverse();

        for index in indices {
            let locked = self
                .current_directory
                .read_unwrap()
                .get_child(index)
                .context("Provided index is out of bounds.")?;
            let child = locked.read_unwrap();
            child.data.delete_entry()?;
            deleted_size += child.data.size;
            drop(child);

            self.current_directory.write_unwrap().remove_child(index);
        }

        Self::apply_upward(&self.current_directory, |node: &mut Node<EntryNode>| {
            node.data.size -= deleted_size;
        });

        Ok(())
    }
}

impl DiskoTree {
    fn process_dir(
        depth: Option<usize>,
        dir_path: &Path,
        state: &mut TreeWalkState,
        children: &mut [jwalk::Result<DirEntry<CustomJWalkClientState>>],
    ) {
        let mut size = match depth {
            // Skip parent directory (./..)
            None => return,
            // The node for the root directory has already been created
            // and is stored as the first parent. In this case, we don't
            // have to do anything with its size, because we know that
            // this directory's self size does not have to be propagated upward.
            Some(0) => EntrySize::default(),
            _ => {
                let Ok(dir_metadata) = fs::metadata(dir_path) else {
                    return;
                };

                let mut dir_node = EntryNode::new(dir_path.to_path_buf(), &dir_metadata);

                // yank the size leaving 0, since we will add the
                // size later during backpropagation
                let size = mem::take(&mut dir_node.size);

                // Create a new node and attach it to the parent stored
                // in the state (which is the directory that contains this entry).
                // Store it in parent to make it visible when jwalk recurses.
                state.parent = Node::create_and_attach_child(&state.parent, dir_node);

                size
            }
        };

        // At this point, `state.parent` actually refers to the current node
        // we are in.
        // This is a bit unintuitive, but simplifies the code quite a bit.

        for child in children {
            let Ok(child) = child else {
                continue;
            };

            if !child.file_type().is_file() {
                continue;
            }

            let Ok(metadata) = child.metadata() else {
                continue;
            };

            let child_size = if state.file_has_been_seen(&metadata) {
                EntrySize::default()
            } else {
                EntrySize::new(&child.path(), &metadata)
            };

            let entry = EntryNode::new_with_size(child.path(), &metadata, child_size);

            size += child_size;
            Node::create_and_attach_child(&state.parent, entry);
        }

        Self::apply_upward(&state.parent, |node: &mut Node<EntryNode>| {
            node.data.size += size;
        });
    }

    /// Apply `mutator` to `node` and its ancestors.
    fn apply_upward<F: FnMut(&mut Node<EntryNode>)>(
        node: &Arc<RwLock<Node<EntryNode>>>,
        mut mutator: F,
    ) {
        let to_root = NodeToRootIterator::new(node.clone());

        to_root.into_iter().for_each(|node| {
            let mut node = node.write_unwrap();

            mutator(&mut node);
        });
    }
}

impl fmt::Display for DiskoTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let root = self.root.read_unwrap();

        write!(f, "{}", root.data)?;
        let children = root.children().to_vec();
        std::mem::drop(root); // Drop the lock on root.

        let Some((last, rest)) = children.split_last() else {
            return Ok(());
        };

        for child in rest {
            let child = child.read_unwrap();
            write!(f, "\n├── {}", child.data)?;
        }
        let last = last.read_unwrap();
        write!(f, "\n└── {}", last.data)
    }
}
