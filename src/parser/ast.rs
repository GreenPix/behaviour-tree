pub use standard::Value;
pub use standard::Operator;

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
