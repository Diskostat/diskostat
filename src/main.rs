
mod backend;

use backend::disko_tree::DiskoTree;

use std::collections::HashMap;
use std::env::current_dir;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use crate::backend::model::node::Node;
use crate::backend::model::tree::Tree;

// TODO: throw here some Arc for multi threading safe usage
#[derive(Default, Debug)]
struct SizeCountingWalkDirState {
    count: i32,
    entries_sizes: HashMap<PathBuf, u128>
}

// impl ClientState for SizeCountingWalkDirState { }


// fn create_disko_tree() -> &'static DiskoTree {
//     Box::leak(Box::new(DiskoTree::new()))
// }

fn main() {



    let mut tree = Tree::new();


    // print!("{}", boxed);

    println!("empty tree: {:?}\n", tree);


    let root_node = tree.set_root(4).expect("Failed to get back root node");

    let node = Node::new(2);

    let newly_created_child_node = Tree::attach_child(root_node, node);

    println!("{tree:?}\n");
    println!("newly created node:\n{newly_created_child_node:?}");



    // println!("Contents of directory:");



    // let disko_tree = create_disko_tree();


    // disko_tree.traverse();


    let current_dir = current_dir().unwrap();
    // println!("Current working directory: {}", current_dir.display());

    // let mut entries = vec![];
    // for entry in fs::read_dir(current_dir).unwrap() {
    //     match entry {
    //         Ok(entry) => {
    //             let path = entry.path();
    //             entries.push(path);
    //         }
    //         Err(err) => {
    //             println!("Failed to read directory entry: {}", err);
    //         }
    //     }
    // }

    // for entry in entries {
    //     let metadata = fs::metadata(entry.clone()).unwrap();
    //     println!("{:?} \t {:?}", entry.file_name().unwrap(), metadata.st_size());
    // }


}
