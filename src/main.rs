
mod backend;

use backend::disko_tree::DiskoTree;

use std::collections::HashMap;
use std::env::current_dir;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use jwalk::WalkDir;
use std::cmp::Ordering;
use jwalk::WalkDirGeneric;
use jwalk::ClientState;
use slab_tree::*;

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



    println!("Contents of directory:");



    let disko_tree = create_disko_tree();


    disko_tree.traverse();


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



    let mut tree = TreeBuilder::new().with_root("hello").build();
    let root_id = tree.root_id().expect("root doesn't exist?");
    let mut hello = tree.get_mut(root_id).unwrap();

    let node = hello.append("world");
    for child in node.as_ref().children() {

    };
    let x = hello
        .append("trees")
        .append("are")
        .append("cool");

}
