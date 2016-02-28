extern crate behaviour_tree;

use std::io::Read;
use std::collections::HashMap;

use behaviour_tree::tree::{BehaviourTreeNode,VisitResult};
use behaviour_tree::standard::{LeavesCollection,Context,StoreKind,Gettable};

impl <'a> Gettable<str,StoreKind> for TestContext<'a> {
    fn get(&self, key: &str) -> Option<&StoreKind> {
        unimplemented!();
    }
}

impl <'a> Context for TestContext<'a> {
    fn insert_value(&mut self, key: String, value: StoreKind) {
        self.inner.insert_value(key,value)
    }

    fn set_value(&mut self, key: &str, value: StoreKind) -> Result<(),()> {
        self.inner.set_value(key,value)
    }
}

impl <'a> TestContext<'a> {
    fn new(test: &'a str) -> TestContext<'a> {
        TestContext {
            inner: HashMap::new(),
            test: test,
        }
    }
}

struct TestContext<'a> {
    inner: HashMap<String,StoreKind>,
    test: &'a str,
}

fn main() {
    println!("Starting process");
    let mut stdin = std::io::stdin();
    let mut string = String::new();
    stdin.read_to_string(&mut string).unwrap();
    let leaves = LeavesCollection::standard();
    let parsed_trees = behaviour_tree::parse(&string, &leaves).unwrap();
    for tree in parsed_trees.iter() {
        println!("Testing tree {}", tree.get_name());
        let mut instance = tree.optimize();
        let mut context = TestContext::new(&string);
        let mut i = 0usize;
        println!("-------- Iteration {} ---------", i);
        while instance.visit(&mut context) == VisitResult::Running {
            i = i + 1;
            println!("-------- Iteration {} ---------", i);
        }
        println!("------- End of tree ----------\n\n");
    }
}
