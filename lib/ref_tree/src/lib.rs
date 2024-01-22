mod model {
    pub mod tree;
    pub mod node;
}

// Reexport Tree & Node for convenience.
pub use model::tree::Tree;
pub use model::node::Node;
