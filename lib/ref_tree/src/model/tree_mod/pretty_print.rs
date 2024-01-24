use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};

use crate::{Node, Tree};

impl<T> Tree<T>
where
    T: Display,
{
    /// # Panics
    pub fn pretty_print(&self) {
        let Some(root) = self.root.clone() else {
            println!("Tree is empty.");
            return;
        };
        println!("{}", root.read().unwrap().data);
        Tree::pretty_print_node(&root, "");
    }

    /// # Panics
    fn pretty_print_node(node: &Arc<RwLock<Node<T>>>, prefix: &str) {
        let node = node
            .read()
            .expect("Could not read node while printing tree.");
        for child in &node.children {
            let is_last = Arc::ptr_eq(child, node.children.last().unwrap());
            println!(
                "{}{}{}",
                prefix,
                if is_last { "└" } else { "├" },
                child.read().unwrap().data
            );
            Tree::pretty_print_node(
                child,
                format!("{}{}", prefix, if is_last { " " } else { "|" }).as_str(),
            );
        }
    }
}
