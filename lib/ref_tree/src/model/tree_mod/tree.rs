use std::sync::Weak;
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
    /// None -> Tree is empty; does not have a root node.
    /// Some -> Tree has at least one node.
    ///
    /// Arc - Multi-thread simultaneous access to the root node.
    /// RwLock
    /// - Ability to read from one thread, but write from others without blocking.
    /// - BE writes when adding size from nodes under root, FE reads with tick/whenever.
    /// - BE also reads when asking for children or going up the three.
    ///
    /// TODO: The docs are out of scope of this lib. Remove or reword it.
    pub(crate) root: Option<Arc<RwLock<Node<T>>>>,
}

// Trait implementations

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Public interface

impl<T> Tree<T> {
    /// Creates an empty tree.
    #[must_use]
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Creates node from given data and puts it to the root of the tree.
    ///
    /// # Errors
    /// - Read them, you're probably doing something wrong.
    pub fn create_node_and_set_root(
        &mut self,
        data: T,
    ) -> Result<Arc<RwLock<Node<T>>>, &'static str> {
        if self.root.is_some() {
            return Err("There is a root already!");
        }
        let root_node = Node::new(data);
        let root_arc = Arc::new(RwLock::new(root_node));
        self.root = Some(root_arc.clone());
        Ok(root_arc)
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
    pub fn attach_child(parent: &Arc<RwLock<Node<T>>>, child: T) -> Arc<RwLock<Node<T>>> {
        let child = {
            let mut parent = parent
                .write()
                .expect("Could not write to parent while attaching child");

            parent.create_and_attach_child(child)
        };

        child
            .write()
            .expect("Writing to newly created child failed when trying to attach its parent")
            .attach_parent(parent);

        // Return newly created and fully connected node.
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
    /// # Errors
    /// - Read them, you're probably doing something wrong.
    ///
    /// When tree's root == node given and given node's
    /// parent is incosistnet.
    pub fn remove_subtree(&mut self, node: &Arc<RwLock<Node<T>>>) -> Result<(), &'static str> {
        if self.root.is_none() {
            return Err("The Tree is empty. Could not remove a subtree from an empty tree.");
        }

        // Check if the tree should remove its root node. And extract
        // the parent why at the to not double read.
        let mut parent: Option<Weak<RwLock<Node<T>>>> = None;
        if let Some(root) = self.root.clone() {
            match (
                Arc::ptr_eq(&root, node),
                node.read()
                    .expect("Could not read node to be removed")
                    .parent
                    .clone(),
            ) {
                (true, None) => {
                    self.root = None;
                    return Ok(());
                }
                (true, Some(_)) => {
                    return Err("Node to be removed is a root of tree, but has a parent.");
                }
                (false, None) => {
                    return Err("Node to be removed is not a root of tree, but has no parent.");
                }
                (false, Some(parent_unwrapped)) => {
                    // Continue.
                    parent = Some(parent_unwrapped);
                }
            }
        }
        let Some(parent) = parent else {
            return Err("This should never happen. I just could not figure \
                        the way how to purify this flow. Michal. #codesmell");
        };

        let Some(parent) = Weak::upgrade(&parent) else {
            return Err("Given node's ");
        };

        Tree::remove_child(&parent, node);

        // Node clean up.
        let mut node = node.write().unwrap();
        node.parent = None;

        Ok(())
    }
}

// Internal convenience functions

impl<T> Tree<T> {
    fn remove_child(parent: &Arc<RwLock<Node<T>>>, child_to_remove: &Arc<RwLock<Node<T>>>) {
        let index = parent
            .read()
            .expect("Failed to read from parent while computing child index")
            .children
            .iter()
            .position(|child| Arc::ptr_eq(child, child_to_remove))
            .expect("The child is missing in parent provided.");

        parent
            .write()
            .expect("Could not write to parent while removing child")
            .children
            .remove(index);
    }

    pub fn iter_to_root_from_node(node: Arc<RwLock<Node<T>>>) -> NodeToRootIterator<T> {
        NodeToRootIterator::new(node)
    }
}
