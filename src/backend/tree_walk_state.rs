use std::{
    fmt::Debug,
    fs,
    sync::{Arc, RwLock},
};

#[cfg(unix)]
use std::{
    collections::{hash_map, HashMap},
    sync::Mutex,
};

use super::entry_node::EntryNode;
use super::node::Node;

#[derive(Clone, Debug, Default)]
pub(crate) struct TreeWalkState {
    pub(crate) parent: Arc<RwLock<Node<EntryNode>>>,
    #[cfg(unix)]
    inodes_unvisited_links: Arc<Mutex<HashMap<u64, u64>>>,
}

impl TreeWalkState {
    pub fn new(root: Arc<RwLock<Node<EntryNode>>>) -> Self {
        Self {
            parent: root,
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

pub(crate) type CustomJWalkClientState = (TreeWalkState, ());
