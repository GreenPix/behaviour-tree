use flat_tree::FlatTree;
use flat_tree::HasChildren;

use tree::non_optimized::*;
use super::OptimizedNode;
use super::OptimizedTree;
use standard::{LeafNodeFactory};

#[derive(Debug)]
pub struct TreeFactory {
    name: String,
    root: NodeFactory,
}

fn optimize_inner(node: &NodeFactory) -> Option<OptimizedNode<'static>> {
    let optimized = match *node {
        NodeFactory::Leaf(ref leaf) => OptimizedNode::Leaf(leaf()),
        NodeFactory::Sequence(_) => OptimizedNode::sequence(None),
        NodeFactory::Selector(_) => OptimizedNode::selector(None),
        NodeFactory::Inverter(_) => OptimizedNode::Inverter,
        NodeFactory::Priority(_) => OptimizedNode::Priority,
        NodeFactory::Blackboard(ref node) => OptimizedNode::blackboard(node.key.clone()),
        NodeFactory::Subtree(_) => panic!("Subtrees are currently unsupported"),
    };
    Some(optimized)
}

impl TreeFactory {
    pub fn new(root: NodeFactory, name: String) -> TreeFactory {
        TreeFactory {
            name: name,
            root: root,
        }
    }

    pub fn instanciate(&self) -> Tree<'static> {
        Tree::new(self.root.instanciate())
    }

    pub fn optimize(&self) -> OptimizedTree<'static> {
        let tree = FlatTree::new(
            &self.root,
            0,
            optimize_inner);
        OptimizedTree{inner: tree}
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

/// Visits all its children in order. If one fails, then return immediatly a failure. If all
/// succeed, then return a success.
///
/// If a child returned "Running", then remember this child and jump directly to him next time the
/// sequence is visited.
///
/// These nodes are typically used to describe a logical sequence of action, when one failure leads
/// to the failure of the whole sequence.
/// For example:
/// 1. Locate door
/// 2. Walk to door
/// 3. Open door
/// 4. Walk through door
#[derive(Debug)]
pub struct SequenceNodeFactory {
    children: Vec<NodeFactory>,
}

impl SequenceNodeFactory {
    pub fn new(children: Vec<NodeFactory>) -> SequenceNodeFactory {
        SequenceNodeFactory {
            children: children,
        }
    }

    pub fn push(&mut self, node: NodeFactory) {
        self.children.push(node);
    }

    pub fn instanciate(&self) -> SequenceNode<'static> {
        let children = self.children.iter().map(|child| child.instanciate()).collect();
        SequenceNode::new(children)
    }
}

/// Counterpart of Sequence: returns Success on the first child returning Success, and return
/// Failure if all children fail.
///
/// This is typically used when a set of actions have the same objective, but those actions are
/// classified by preference.
#[derive(Debug)]
pub struct SelectorNodeFactory {
    children: Vec<NodeFactory>,
}

impl SelectorNodeFactory {
    pub fn new(children: Vec<NodeFactory>) -> SelectorNodeFactory {
        SelectorNodeFactory {
            children: children,
        }
    }

    pub fn push(&mut self, node: NodeFactory) {
        self.children.push(node);
    }

    pub fn instanciate(&self) -> SelectorNode<'static> {
        let children = self.children.iter().map(|child| child.instanciate()).collect();
        SelectorNode::new(children)
    }
}

/// Same as Sequence, but do not remember the last running child and revisit all children
#[derive(Debug)]
pub struct PriorityNodeFactory {
    children: Vec<NodeFactory>,
}

impl PriorityNodeFactory {
    pub fn new(children: Vec<NodeFactory>) -> PriorityNodeFactory {
        PriorityNodeFactory{children: children}
    }

    pub fn push(&mut self, node: NodeFactory) {
        self.children.push(node);
    }

    pub fn instanciate(&self) -> PriorityNode<'static> {
        let children = self.children.iter().map(|child| child.instanciate()).collect();
        PriorityNode::new(children)
    }
}

/// Checks for the presence of a key in the context. Immediatly return failure if it doesn't.
#[derive(Debug)]
pub struct BlackboardNodeFactory {
    child: Box<NodeFactory>,
    key: String,
}

impl BlackboardNodeFactory {
    pub fn new(child: Box<NodeFactory>, key: String) -> BlackboardNodeFactory {
        BlackboardNodeFactory{child: child, key: key}
    }

    pub fn instanciate(&self) -> BlackboardNode<'static> {
        unimplemented!();
    }
}

/// Inverts the output of the child
#[derive(Debug)]
pub struct InverterNodeFactory {
    child: Box<NodeFactory>,
}

impl InverterNodeFactory {
    pub fn new(child: Box<NodeFactory>) -> InverterNodeFactory {
        InverterNodeFactory{child: child}
    }

    pub fn instanciate(&self) -> InverterNode<'static> {
        let child = Box::new(self.child.instanciate());
        InverterNode::new(child)
    }
}

#[derive(Debug)]
pub enum NodeFactory {
    Leaf(LeafNodeFactory),
    Sequence(SequenceNodeFactory),
    Priority(PriorityNodeFactory),
    Selector(SelectorNodeFactory),
    Blackboard(BlackboardNodeFactory),
    Inverter(InverterNodeFactory),
    Subtree(String),
}

impl NodeFactory {
    pub fn instanciate<'a>(&self) -> Node<'a> {
        match *self {
            NodeFactory::Leaf(ref node_factory) => Node::Leaf(node_factory()),
            NodeFactory::Sequence(ref node) => Node::Sequence(node.instanciate()),
            NodeFactory::Priority(ref node) => Node::Priority(node.instanciate()),
            NodeFactory::Selector(ref node) => Node::Selector(node.instanciate()),
            NodeFactory::Blackboard(ref node) => Node::Blackboard(node.instanciate()),
            NodeFactory::Inverter(ref node) => Node::Inverter(node.instanciate()),
            NodeFactory::Subtree(ref name) => panic!("Trying to instanciate an unlinked subtree {}", name),
        }
    }

    pub fn new_leaf(factory: LeafNodeFactory) -> NodeFactory {
        NodeFactory::Leaf(factory)
    }

    pub fn new_sequence(children: Vec<NodeFactory>) -> NodeFactory {
        NodeFactory::Sequence(SequenceNodeFactory::new(children))
    }

    pub fn new_selector(children: Vec<NodeFactory>) -> NodeFactory {
        NodeFactory::Selector(SelectorNodeFactory::new(children))
    }

    pub fn new_priority(children: Vec<NodeFactory>) -> NodeFactory {
        NodeFactory::Priority(PriorityNodeFactory::new(children))
    }

    pub fn new_inverter(child: Box<NodeFactory>) -> NodeFactory {
        NodeFactory::Inverter(InverterNodeFactory::new(child))
    }

    pub fn new_subtree(name: String) -> NodeFactory {
        NodeFactory::Subtree(name)
    }
}

impl HasChildren for NodeFactory {
    fn get_children(&self) -> &[NodeFactory] {
        match *self {
            NodeFactory::Leaf(_) => &[],
            NodeFactory::Sequence(ref node) => &node.children,
            NodeFactory::Priority(ref node) => &node.children,
            NodeFactory::Selector(ref node) => &node.children,
            NodeFactory::Blackboard(ref node) => ::ref_slice::ref_slice(&node.child),
            NodeFactory::Inverter(ref node) => ::ref_slice::ref_slice(&node.child),
            NodeFactory::Subtree(ref name) => panic!("Trying to instanciate an unlinked subtree {}", name),
        }
    }
}
