mod model {
    pub mod tree_mod {
        pub mod tree;
        mod tests;
    }
    pub mod node_mod {
        pub mod node;
        pub mod node_to_root_iterator;
        mod tests;
    }
}

// Reexport Tree & Node for convenience.
pub use model::tree_mod::tree::Tree;
pub use model::node_mod::node::Node;
pub use model::node_mod::node_to_root_iterator::NodeToRootIterator;
