
mod backend;

use backend::disko_tree::DiskoTree;

use std::collections::HashMap;
use std::env::current_dir;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use crate::backend::disko_tree;
use crate::backend::model::entry_node::EntryNode;

// TODO: throw here some Arc for multi threading safe usage
#[derive(Default, Debug)]
struct SizeCountingWalkDirState {
    count: i32,
    entries_sizes: HashMap<PathBuf, u128>
}

// impl ClientState for SizeCountingWalkDirState { }


fn create_disko_tree() -> &'static DiskoTree {
    Box::leak(Box::new(DiskoTree::new()))
}

fn main() {

    // let current_dir = current_dir().unwrap();

    // let mut tree = Tree::new();
    // tree.create_and_set_root(EntryNode::new_dir(&current_dir).unwrap());
    // tree.pretty_print();


    // return;

    // println!("Contents of directory:");



    let disko_tree = create_disko_tree();
    disko_tree.traverse();

    let tree = disko_tree.get_tree();
    // tree.read().unwrap().pretty_print();

    let root = { tree.read().unwrap().get_root().unwrap() };

    let data = {
        root.read().unwrap().data.clone()
    };

    println!("Total size of ./: {}", data.size);



    // println!("{:?}", disko_tree.get_tree().read());



    // disko_tree.traverse();


    // let current_dir = current_dir().unwrap();
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
