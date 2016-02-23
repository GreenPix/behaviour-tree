extern crate behaviour_tree;

use std::io::Read;
use std::collections::HashMap;

use behaviour_tree::tree::{Context,BehaviourTreeNode,VisitResult};
use behaviour_tree::standard::LEAVES_COLLECTION;

fn main() {
    println!("Starting process");
    let mut stdin = std::io::stdin();
    let mut string = String::new();
    stdin.read_to_string(&mut string).unwrap();
    let test = vec![&LEAVES_COLLECTION];
    let parsed_trees = behaviour_tree::parse(&string, &(test[..])).unwrap();
    for tree in parsed_trees.iter() {
        println!("Testing tree {}", tree.get_name());
        let mut instance = tree.optimize();
        let mut context = Context::new(HashMap::new());
        let mut i = 0usize;
        println!("-------- Iteration {} ---------", i);
        while instance.visit(&mut context) == VisitResult::Running {
            i = i + 1;
            println!("-------- Iteration {} ---------", i);
        }
        println!("------- End of tree ----------\n\n");
    }
}
