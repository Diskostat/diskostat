#![warn(clippy::pedantic)]

pub mod tree_mod {
    mod pretty_print;
    mod tests;
    pub mod tree;
}
pub mod node_mod {
    pub mod node;
    pub mod node_to_root_iterator;
    mod tests;
}

// Reexport Tree & Node for convenience.
pub use node_mod::node::Node;
pub use node_mod::node_to_root_iterator::NodeToRootIterator;
pub use tree_mod::tree::Tree;

use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait RwLockExt<T> {
    fn read_unwrap(&self) -> RwLockReadGuard<'_, T>;
    fn write_unwrap(&self) -> RwLockWriteGuard<'_, T>;
}

impl<T> RwLockExt<T> for RwLock<T> {
    fn read_unwrap(&self) -> RwLockReadGuard<'_, T> {
        self.read()
            .expect("RwLock should not be poisoned (writer panicked while holding lock?)")
    }
    fn write_unwrap(&self) -> RwLockWriteGuard<'_, T> {
        self.write()
            .expect("RwLock should not be poisoned (writer panicked while holding lock?)")
    }
}
