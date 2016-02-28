use flat_tree::FlatTree;
use flat_tree::HasChildren;

use tree::non_optimized::*;
use super::OptimizedNode;
use super::OptimizedTree;
use super::LeafNode;
use super::{LeafNodeFactory};

#[derive(Debug)]
pub struct TreeFactory<F> {
    name: String,
    root: NodeFactory<F>,
}

fn optimize_inner<F: LeafNodeFactory>(node: &NodeFactory<F>)
-> Option<OptimizedNode<<F as LeafNodeFactory>::Output>> {
    let optimized = match *node {
        NodeFactory::Leaf(ref leaf) => OptimizedNode::Leaf(LeafNode::new(leaf.instanciate())),
        NodeFactory::Sequence(_) => OptimizedNode::sequence(None),
        NodeFactory::Selector(_) => OptimizedNode::selector(None),
        NodeFactory::Inverter(_) => OptimizedNode::Inverter,
        NodeFactory::Priority(_) => OptimizedNode::Priority,
        NodeFactory::Subtree(_) => panic!("Subtrees are currently unsupported"),
    };
    Some(optimized)
}

impl <F> TreeFactory<F> {
    pub fn new(root: NodeFactory<F>, name: String) -> TreeFactory<F> {
        TreeFactory {
            name: name,
            root: root,
        }
    }

    pub fn instanciate(&self) -> Tree<F::Output>
    where F: LeafNodeFactory {
        Tree::new(self.root.instanciate())
    }

    pub fn optimize(&self) -> OptimizedTree<F::Output>
    where F: LeafNodeFactory {
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
pub struct SequenceNodeFactory<F> {
    children: Vec<NodeFactory<F>>,
}

impl <F> SequenceNodeFactory<F> {
    pub fn new(children: Vec<NodeFactory<F>>) -> SequenceNodeFactory<F> {
        SequenceNodeFactory {
            children: children,
        }
    }

    pub fn push(&mut self, node: NodeFactory<F>) {
        self.children.push(node);
    }

    pub fn instanciate(&self) -> SequenceNode<F::Output>
    where F: LeafNodeFactory {
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
pub struct SelectorNodeFactory<F> {
    children: Vec<NodeFactory<F>>,
}

impl <F> SelectorNodeFactory<F> {
    pub fn new(children: Vec<NodeFactory<F>>) -> SelectorNodeFactory<F> {
        SelectorNodeFactory {
            children: children,
        }
    }

    pub fn push(&mut self, node: NodeFactory<F>) {
        self.children.push(node);
    }

    pub fn instanciate(&self) -> SelectorNode<F::Output>
    where F: LeafNodeFactory {
        let children = self.children.iter().map(|child| child.instanciate()).collect();
        SelectorNode::new(children)
    }
}

/// Same as Sequence, but do not remember the last running child and revisit all children
#[derive(Debug)]
pub struct PriorityNodeFactory<F> {
    children: Vec<NodeFactory<F>>,
}

impl <F> PriorityNodeFactory<F> {
    pub fn new(children: Vec<NodeFactory<F>>) -> PriorityNodeFactory<F> {
        PriorityNodeFactory{children: children}
    }

    pub fn push(&mut self, node: NodeFactory<F>) {
        self.children.push(node);
    }

    pub fn instanciate(&self) -> PriorityNode<F::Output>
    where F: LeafNodeFactory {
        let children = self.children.iter().map(|child| child.instanciate()).collect();
        PriorityNode::new(children)
    }
}

/// Inverts the output of the child
#[derive(Debug)]
pub struct InverterNodeFactory<F> {
    child: Box<NodeFactory<F>>,
}

impl <F> InverterNodeFactory<F> {
    pub fn new(child: Box<NodeFactory<F>>) -> InverterNodeFactory<F> {
        InverterNodeFactory{child: child}
    }

    pub fn instanciate(&self) -> InverterNode<F::Output>
    where F: LeafNodeFactory {
        let child = Box::new(self.child.instanciate());
        InverterNode::new(child)
    }
}

#[derive(Debug)]
pub enum NodeFactory<F> {
    Leaf(F),
    Sequence(SequenceNodeFactory<F>),
    Priority(PriorityNodeFactory<F>),
    Selector(SelectorNodeFactory<F>),
    Inverter(InverterNodeFactory<F>),
    Subtree(String),
}

impl <F> NodeFactory<F> {
    pub fn instanciate(&self) -> Node<F::Output>
    where F: LeafNodeFactory {
        match *self {
            NodeFactory::Leaf(ref node_factory) => Node::Leaf(LeafNode::new(node_factory.instanciate())),
            NodeFactory::Sequence(ref node) => Node::Sequence(node.instanciate()),
            NodeFactory::Priority(ref node) => Node::Priority(node.instanciate()),
            NodeFactory::Selector(ref node) => Node::Selector(node.instanciate()),
            NodeFactory::Inverter(ref node) => Node::Inverter(node.instanciate()),
            NodeFactory::Subtree(ref name) => panic!("Trying to instanciate an unlinked subtree {}", name),
        }
    }

    pub fn new_leaf(factory: F) -> NodeFactory<F> {
        NodeFactory::Leaf(factory)
    }

    pub fn new_sequence(children: Vec<NodeFactory<F>>) -> NodeFactory<F> {
        NodeFactory::Sequence(SequenceNodeFactory::new(children))
    }

    pub fn new_selector(children: Vec<NodeFactory<F>>) -> NodeFactory<F> {
        NodeFactory::Selector(SelectorNodeFactory::new(children))
    }

    pub fn new_priority(children: Vec<NodeFactory<F>>) -> NodeFactory<F> {
        NodeFactory::Priority(PriorityNodeFactory::new(children))
    }

    pub fn new_inverter(child: Box<NodeFactory<F>>) -> NodeFactory<F> {
        NodeFactory::Inverter(InverterNodeFactory::new(child))
    }

    pub fn new_subtree(name: String) -> NodeFactory<F> {
        NodeFactory::Subtree(name)
    }
}

impl <F> HasChildren for NodeFactory<F> {
    fn get_children(&self) -> &[NodeFactory<F>] {
        match *self {
            NodeFactory::Leaf(_) => &[],
            NodeFactory::Sequence(ref node) => &node.children,
            NodeFactory::Priority(ref node) => &node.children,
            NodeFactory::Selector(ref node) => &node.children,
            NodeFactory::Inverter(ref node) => ::ref_slice::ref_slice(&node.child),
            NodeFactory::Subtree(ref name) => panic!("Trying to instanciate an unlinked subtree {}", name),
        }
    }
}
