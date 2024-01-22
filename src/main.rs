mod backend;

use backend::disko_tree::DiskoTree;

fn create_disko_tree() -> &'static DiskoTree {
    Box::leak(Box::new(DiskoTree::new()))
}

fn main() {

    let disko_tree = create_disko_tree();
    disko_tree.traverse();

    let tree = disko_tree.get_tree();
    // tree.read().unwrap().pretty_print();

    let root = { tree.read().unwrap().get_root().unwrap() };

    let data = {
        root.read().unwrap().data.clone()
    };

    println!("Total size of .: {}", data.size);

    // TODO: Prety print the tree with sizes
}
