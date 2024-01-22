#[cfg(test)]
use crate::{Tree, Node, NodeToRootIterator};

#[test]
fn test_node_new() {
    let node = Node::new(0);
    assert_eq!(node.data, 0);
    assert!(node.children.is_empty());
    assert!(node.parent.is_none());
}

#[test]
fn test_node_attach_child() {
    let mut node = Node::new(0);
    let child_node = Node::new(1);
    let child = node.attach_child(child_node);
    assert_eq!(node.children.len(), 1);
    assert_eq!(node.children[0].read().unwrap().data, 1);
    assert_eq!(child.read().unwrap().data, 1);
}

#[test]
fn test_node_get_children() {
    let mut node = Node::new(0);
    let child = Node::new(1);
    let child_arc = node.attach_child(child);
    let children = node.get_children();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].read().unwrap().data, 1);
    assert_eq!(child_arc.read().unwrap().data, 1);
}

#[test]
fn test_node_child_writability() {
    let mut node = Node::new(0);
    let child_node = Node::new(1);
    let child = node.attach_child(child_node);
    {
        let mut child = child.write().unwrap();
        child.data = 2;
    } // Drop write lock.
    assert_eq!(child.read().unwrap().data, 2);
    assert_eq!(node.get_children().first().unwrap().read().unwrap().data, 2);
}


#[test]
fn test_node_to_tree_iterator() {
    let mut tree = Tree::new();
    let root_node = tree.create_and_set_root(0).unwrap();
    let child_node = Node::new(1);
    let child = Tree::attach_child(root_node.clone(), child_node);
    let child2_node = Node::new(2);
    let child2 = Tree::attach_child(child.clone(), child2_node);

    let mut iter = NodeToRootIterator::new(child2);

    assert_eq!(iter.next().expect("next is none").read().unwrap().data, 1);
    assert_eq!(iter.next().unwrap().read().unwrap().data, 0);
    assert!(iter.next().is_none());
}
