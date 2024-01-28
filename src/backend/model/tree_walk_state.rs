use std::{
    fmt::{Debug, Formatter},
    fs,
    sync::{Arc, RwLock},
};

#[cfg(unix)]
use std::{
    collections::{hash_map, HashMap},
    sync::Mutex,
};

use super::entry_node::EntryNode;
use ref_tree::{Node, Tree};

#[derive(Clone)]
pub(crate) enum TreeWalkAncestor {
    Tree(Arc<RwLock<Tree<EntryNode>>>),
    Parent(Arc<RwLock<Node<EntryNode>>>),
}

#[derive(Clone)]
pub(crate) struct TreeWalkState {
    pub(crate) ancestor: TreeWalkAncestor,
    #[cfg(unix)]
    inodes_unvisited_links: Arc<Mutex<HashMap<u64, u64>>>,
}

impl TreeWalkState {
    pub fn new(tree: Arc<RwLock<Tree<EntryNode>>>) -> Self {
        Self {
            ancestor: TreeWalkAncestor::Tree(tree),
            #[cfg(unix)]
            inodes_unvisited_links: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[cfg(unix)]
    pub fn file_has_been_seen(&mut self, metadata: &fs::Metadata) -> bool {
        use std::os::unix::fs::MetadataExt;

        let inode = metadata.ino();
        let links = metadata.nlink();
        if links > 1 {
            let mut inodes_unvisited_links = self
                .inodes_unvisited_links
                .lock()
                .expect("Failed to lock inode map.");
            match inodes_unvisited_links.entry(inode) {
                hash_map::Entry::Occupied(mut hash_map_entry) => {
                    let count = hash_map_entry.get_mut();
                    if *count == 1 {
                        // The final time we are seeing this inode.
                        hash_map_entry.remove();
                    } else {
                        *count -= 1;
                    }
                    true
                }
                // The first time we are seeing this inode.
                hash_map::Entry::Vacant(hash_map_entry) => {
                    hash_map_entry.insert(links - 1);
                    false
                }
            }
        } else {
            false
        }
    }

    /// Seems like this could also be done for windows using:
    /// https://doc.rust-lang.org/std/os/windows/fs/trait.MetadataExt.html#tymethod.number_of_links
    /// https://doc.rust-lang.org/std/os/windows/fs/trait.MetadataExt.html#tymethod.file_index
    /// However, it is nightly-only experimental API
    #[cfg(not(unix))]
    pub fn file_has_been_seen(&mut self, _metadata: &fs::Metadata) -> bool {
        false
    }
}

impl Default for TreeWalkState {
    fn default() -> Self {
        Self {
            ancestor: TreeWalkAncestor::Tree(Arc::new(RwLock::new(Tree::new()))),
            #[cfg(unix)]
            inodes_unvisited_links: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Debug for TreeWalkState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.ancestor {
            TreeWalkAncestor::Tree(_) => write!(f, "TreeWalkState {{ ancestor: Tree }}"),
            TreeWalkAncestor::Parent(_) => write!(f, "TreeWalkState {{ ancestor: Parent }}"),
        }
    }
}

pub(crate) type CustomJWalkClientState = (TreeWalkState, ());
