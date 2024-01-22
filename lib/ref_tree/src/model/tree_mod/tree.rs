use std::sync::{Arc, RwLock};

use crate::Node;
use crate::NodeToRootIterator;

/// Tree made out of references. Multi-threaded.
///
/// To get a certain node, you have to traverse all the tree -> do NOT do it.
///
#[derive(Debug)]
pub struct Tree<T> {
    /// Root node of the three
    ///
    /// Option
    /// None -> Tree is empty; does not have a root node
    /// Some -> Tree has at least one node
    ///
    /// Arc - multi-thread simultaneous access to the root node
    /// RwLock
    /// - ability to read from one thread, but write from other without blocking
    /// - BE writes when adding size from nodes under root, FE reads with tick/whenever
    pub(crate) root: Option<Arc<RwLock<Node<T>>>>,
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Tree<T> {
    /// Creates an empty tree.
    #[must_use]
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn set_root_node(&mut self, node: Node<T>) -> Arc<RwLock<Node<T>>> {
        let root_arc = Arc::new(RwLock::new(node));
        self.root = Some(root_arc.clone());
        root_arc
    }

    /// Creates node from given data and puts it to the root of the tree.
    /// Returns: None if tree already has a root node.
    /// Returns: Created node from given data.
    pub fn create_and_set_root(&mut self, data: T) -> Option<Arc<RwLock<Node<T>>>> {
        if self.root.is_some() {
            return None;
        }
        let root_node = Node::new(data);
        let root_arc = Arc::new(RwLock::new(root_node));
        self.root = Some(root_arc.clone());
        Some(root_arc)
    }

    #[must_use]
    pub fn get_root(&self) -> Option<Arc<RwLock<Node<T>>>> {
        self.root.clone()
    }

    /// Attaches given node (child) to tree in which parent is stored.
    ///
    /// Connects both parent -> child and child -> parent.
    ///
    /// Returns newly created space in which child now is stored.
    ///
    /// # Panics
    /// When could not write to parrent, see `RwLock`.
    pub fn attach_child(parent: &Arc<RwLock<Node<T>>>, child: Node<T>) -> Arc<RwLock<Node<T>>> {
        // Connect child to parent.
        let mut child = child;
        child.parent = Some(Arc::downgrade(parent));

        // Borrow parent for writing, create and attach child.
        let child = {
            let mut parent = parent
                .write()
                .expect("Could not write to parent while attaching child");

            parent.attach_child(child)
        };

        // Return newly created and connected node.
        child
    }

    /// Data/node will be freed from memory when the callee drops the
    /// reference if last (see Arc).  This function just drops the
    /// reference of the parent of this tree and sets root to node if
    /// the node is the root of this tree.
    ///
    /// # Panics
    ///
    /// When could not read the node given, when asking for parent,
    /// see `RwLock`.
    ///
    /// When tree's root == node given and given node's
    /// parent is incosistnet.
    pub fn remove_subtree(&mut self, node: &Arc<RwLock<Node<T>>>) {
        // TODO: check if tree has root before removing node?

        if let Some(root) = self.root.clone() {
            match (
                Arc::ptr_eq(&root, node),
                node.clone()
                    .read()
                    .expect("Could not read node while removing subtree.")
                    .parent
                    .clone(),
            ) {
                (true, None) => {
                    self.root = None;
                    return;
                }
                (true, Some(_)) => {
                    panic!("Node to be removed is a root of tree, but has a parent. This should not happen.");
                }
                (false, None) => {
                    panic!("Node to be removed is not a root of tree, but has no parent. This should not happen.");
                }
                (false, Some(_)) => {
                    // Continue.
                }
            }
        }

        // Remove node from parent.
        let Some(parent) = node.read().unwrap().parent.clone() else {
            panic!("Node to be removed has no parent. This should not happen.");
        };

        let parent = parent
            .upgrade()
            .expect("Could not upgrade parent of node to be removed. This should not happen.");
        Tree::remove_children(&parent, node);

        // Node clean up.
        let mut node = node.write().unwrap();
        node.parent = None;
    }

    fn remove_children(parent: &Arc<RwLock<Node<T>>>, child_to_remove: &Arc<RwLock<Node<T>>>) {
        let index = {
            let mut index_ret = 0;
            for (index, child) in parent.write().unwrap().children.iter().enumerate() {
                if Arc::ptr_eq(child, child_to_remove) {
                    index_ret = index;
                    break;
                }
            }
            index_ret
        };
        let mut parent = parent
            .write()
            .expect("Could not write to parent while removing child");
        parent.children.remove(index);
    }

    pub fn iter_to_root_from_node(node: Arc<RwLock<Node<T>>>) -> NodeToRootIterator<T> {
        NodeToRootIterator::new(node)
    }
}

// TODO integrate in main crate
// impl Tree<EntryNode> {
//     pub(crate) fn pretty_print(&self)  {
//         let Some(root) = self.root.clone() else {
//             println!("Tree is empty.");
//             return;
//         };
//         Tree::pretty_print_node(0, 0, true, root);
//     }

//     fn pretty_print_node(depth: u32, plunge_diff: u32, is_last: bool, node: Arc<RwLock<Node<EntryNode>>>) {
//         let node = node.read().expect("Could not read node while printing tree.");
//         println!("{}{}", Self::prefix(depth, plunge_diff, is_last), node.pretty());
//         let plunge_diff = if is_last && depth == plunge_diff { plunge_diff + 1} else { plunge_diff };
//         for child in node.children.iter() {
//             let is_last = Arc::ptr_eq(child, node.children.last().unwrap());
//             Tree::pretty_print_node(depth + 1, plunge_diff, is_last, child.clone());
//         }
//     }

//     fn prefix(depth: u32, plunge_diff: u32, is_last: bool) -> String {
//         if depth == 0 { return "".to_string(); }
//         let mut result = String::new();
//         for _ in 0..plunge_diff {
//             result.push(' ');
//         }
//         for _ in 0..(depth - plunge_diff) {
//             result.push('|');
//         }
//         result.push_str(if is_last { "└" } else { "├" });
//         result
//     }
// }
