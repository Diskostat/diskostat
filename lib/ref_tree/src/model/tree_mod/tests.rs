#[cfg(test)]
use crate::Tree;

#[test]
fn test_set_root() {
    let mut tree = Tree::new();
    let root = tree.create_node_and_set_root(0);
    assert!(root.is_ok());
    assert!(tree.root.is_some());
    assert_eq!(tree.root.clone().unwrap().read().unwrap().data, 0);
    assert!(tree
        .root
        .clone()
        .unwrap()
        .read()
        .unwrap()
        .children
        .is_empty());
}

#[test]
fn test_get_root() {
    let mut tree = Tree::new();
    let root = tree.create_node_and_set_root(0).unwrap();
    let root2 = tree.get_root().unwrap();
    assert_eq!(root.read().unwrap().data, root2.read().unwrap().data);
}

#[test]
fn test_attach_child() {
    let mut tree = Tree::new();
    let root = tree.create_node_and_set_root(0).unwrap();
    let child = Tree::attach_child(&root, 1);
    assert_eq!(root.read().unwrap().children.len(), 1);
    assert_eq!(
        root.read()
            .unwrap()
            .children
            .first()
            .unwrap()
            .read()
            .unwrap()
            .data,
        1
    );
    assert_eq!(
        child
            .read()
            .unwrap()
            .parent
            .as_ref()
            .unwrap()
            .upgrade()
            .unwrap()
            .read()
            .unwrap()
            .data,
        0
    );
    assert_eq!(child.read().unwrap().data, 1);
}

#[test]
fn test_remove_subtree() {
    let mut tree = Tree::new();
    let root = tree.create_node_and_set_root(0).unwrap();
    let child = Tree::attach_child(&root, 1);
    assert!(tree.remove_subtree(&child).is_ok());
    assert!(child.read().unwrap().parent.is_none());
    assert!(child.read().unwrap().children.is_empty());
    assert!(root.read().unwrap().children.is_empty());
}

#[test]
fn test_remove_subree_with_two_layers() {
    let mut tree = Tree::new();
    let root = tree.create_node_and_set_root(0).unwrap();
    let child = Tree::attach_child(&root, 1);
    let child2 = Tree::attach_child(&child, 2);

    assert!(tree.remove_subtree(&child).is_ok());

    assert!(child.read().unwrap().parent.is_none());
    assert!(root.read().unwrap().get_children().is_empty());
    assert!(root.read().unwrap().children.is_empty());

    // remove does to modify childern of given node
    assert!(!child.read().unwrap().children.is_empty());

    // romve does not remove parent of layers down the route
    assert!(child2.read().unwrap().parent.is_some());

    // remove does not remove children of layers down the route
    // weaks upstream and arcs take care of freeing the memory
    // someone might still use it
    assert!(child2.read().unwrap().children.is_empty());
}
