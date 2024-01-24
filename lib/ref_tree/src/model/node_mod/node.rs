use std::{
    fmt::Debug,
    sync::{Arc, RwLock, Weak},
};

#[derive(Debug)]
pub struct Node<T>
where
    T: Clone,
{
    /// empty vec -> leaf node (only in tree struct, not in fs,
    /// i.e. can be empty dir)
    pub(crate) children: Vec<Arc<RwLock<Node<T>>>>,

    pub data: T,

    /// None -> root node
    /// Weak -> prevent reference cycles
    pub(crate) parent: Option<Weak<RwLock<Node<T>>>>,
}

impl<T> Node<T>
where
    T: Clone,
{
    /// Creates not connected Node.
    pub fn new(data: T) -> Self {
        Self {
            children: vec![],
            data,
            parent: None,
        }
    }

    /// Attaches given node to this node as a child.
    /// Does NOT attach parent.
    ///
    /// Consumes given Node. Returns newly created Arc-RwlLock for the new node.
    pub fn attach_child(&mut self, node: Node<T>) -> Arc<RwLock<Node<T>>> {
        let other_node_arc = Arc::new(RwLock::new(node));
        self.children.push(other_node_arc.clone());

        other_node_arc
    }

    pub fn get_children(&self) -> Vec<Arc<RwLock<Node<T>>>> {
        self.children.clone()
    }

    pub fn get_children_data(&self) -> Vec<(T, usize)> {
        self.children
            .iter()
            .enumerate()
            .map(|(i, c)| (c.clone().read().unwrap().data.clone(), i))
            .collect()
    }
}

// TODO: integrate in main crate

// impl Node<EntryNode> {
//     // TODO: Create trait
//     // TODO: User formatter as Display does
//     pub(crate) fn pretty(&self) -> String {
//         format!("{:<20} • {}", self.data.name, self.data.size)
//     }
// }
