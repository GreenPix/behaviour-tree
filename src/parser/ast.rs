use parser::Value;

pub enum Node {
    Sequence(Vec<Node>),
    Selector(Vec<Node>),
    Priority(Vec<Node>),
    Leaf(String,Option<Value>),
    Inverter(Box<Node>),
}

pub struct Tree {
    pub name: String,
    pub root: Node,
}
