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
