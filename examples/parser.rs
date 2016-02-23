extern crate behaviour_tree;

use std::env;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

use behaviour_tree::tree::{Context,BehaviourTreeNode};
use behaviour_tree::standard::LEAVES_COLLECTION;

fn main() {
    let mut args = env::args_os();
    args.next();
    for filename in args {
        let mut file = match File::open(filename) {
            Ok(file) => file,
            Err(e) => {
                println!("Error {}", e);
                continue;
            }
        };
        let mut string = String::new();
        file.read_to_string(&mut string).unwrap();
        let leaves = vec![&LEAVES_COLLECTION];
        let parsed_trees = behaviour_tree::parse(&string, &leaves[..]).unwrap();
        for tree in parsed_trees.iter() {
            println!("Testing tree {}", tree.get_name());
            let mut instance = tree.instanciate();
            let mut context = Context::new(HashMap::new());
            instance.visit(&mut context);
        }
    }
}
