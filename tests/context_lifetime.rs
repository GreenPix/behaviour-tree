extern crate behaviour_tree;

use std::collections::HashMap;
use behaviour_tree::tree::{BehaviourTreeNode};
use behaviour_tree::standard::{LeavesCollection,Context,StoreKind,Gettable};

const TREE: &'static str = r#"
tree test {
    print_text("Hello World")
}
"#;
impl <'a> Gettable<str,StoreKind> for TestContext<'a> {
    fn get(&self, key: &str) -> Option<&StoreKind> {
        self.inner.get(key)
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

#[allow(unused)]
struct TestContext<'a> {
    inner: HashMap<String,StoreKind>,
    test: &'a str,
}

#[test]
fn test() {
    let leaves = LeavesCollection::standard();
    let parsed_trees = behaviour_tree::parse(TREE, &leaves).unwrap();
    for tree in parsed_trees.iter() {
        println!("Testing tree {}", tree.get_name());
        let mut instance = tree.instanciate();
        let string: &'static str = "This one works";
        //let string: &str = &String::from("This one doesn't");
        let mut context = TestContext::new(string);
        instance.visit(&mut context);
    }
}
