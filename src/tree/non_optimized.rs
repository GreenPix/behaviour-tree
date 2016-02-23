use std::fmt::{self,Debug,Formatter};

use super::{VisitResult,Context,BehaviourTreeNode,LeafNode};

#[derive(Debug)]
pub struct Tree<'a> {
    root: Node<'a>,
}

impl <'a> Tree<'a> {
    pub fn new(root: Node<'a>) -> Tree<'a> {
        Tree {
            root: root,
        }
    }
}

impl <'a> BehaviourTreeNode for Tree<'a> {
    fn visit(&mut self, context: &mut Context) -> VisitResult {
        self.root.visit(context)
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
pub struct SequenceNode<'a> {
    running: Option<usize>,
    children: Vec<Node<'a>>,
}

impl <'a> BehaviourTreeNode for SequenceNode<'a> {
    fn visit(&mut self, context: &mut Context) -> VisitResult {
        // If we were running, start again where we left
        let start = self.running.take().unwrap_or(0);
        for (pos, child) in self.children[start..].iter_mut().enumerate() {
            let result = child.visit(context);
            match result {
                VisitResult::Failure => return VisitResult::Failure,
                VisitResult::Running => {
                    self.running = Some(start + pos);
                    return VisitResult::Running;
                }
                VisitResult::Success => {}
            }
        }
        VisitResult::Success
    }
}

impl <'a> SequenceNode<'a> {
    pub fn new(children: Vec<Node<'a>>) -> SequenceNode<'a> {
        SequenceNode {
            running: None,
            children: children,
        }
    }

    #[allow(dead_code)]
    // Kept just in case ...
    pub fn push(&mut self, node: Node<'a>) {
        self.children.push(node);
    }
}

/// Counterpart of Sequence: returns Success on the first child returning Success, and return
/// Failure if all children fail.
///
/// This is typically used when a set of actions have the same objective, but those actions are
/// classified by preference.
#[derive(Debug)]
pub struct SelectorNode<'a> {
    running: Option<usize>,
    children: Vec<Node<'a>>,
}

impl <'a> BehaviourTreeNode for SelectorNode<'a> {
    fn visit(&mut self, context: &mut Context) -> VisitResult {
        // If we were running, start again where we left
        let start = self.running.take().unwrap_or(0);
        for (pos, child) in self.children[start..].iter_mut().enumerate() {
            let result = child.visit(context);
            match result {
                VisitResult::Success => return VisitResult::Failure,
                VisitResult::Running => {
                    self.running = Some(start + pos);
                    return VisitResult::Running;
                }
                VisitResult::Failure => {}
            }
        }
        VisitResult::Failure
    }
}

impl <'a> SelectorNode<'a> {
    pub fn new(children: Vec<Node<'a>>) -> SelectorNode<'a> {
        SelectorNode {
            running: None,
            children: children,
        }
    }

    #[allow(dead_code)]
    // Kept just in case ...
    pub fn push(&mut self, node: Node<'a>) {
        self.children.push(node);
    }
}

/// Same as Sequence, but do not remember the last running child and revisit all children
#[derive(Debug)]
pub struct PriorityNode<'a> {
    children: Vec<Node<'a>>,
}

impl <'a> BehaviourTreeNode for PriorityNode<'a> {
    fn visit(&mut self, context: &mut Context) -> VisitResult {
        for child in self.children.iter_mut() {
            let result = child.visit(context);
            match result {
                VisitResult::Failure => return VisitResult::Failure,
                VisitResult::Running => return VisitResult::Running,
                VisitResult::Success => {}
            }
        }
        VisitResult::Success
    }
}

impl <'a> PriorityNode<'a> {
    pub fn new(children: Vec<Node<'a>>) -> PriorityNode<'a> {
        PriorityNode{children: children}
    }

    #[allow(dead_code)]
    // Kept just in case ...
    pub fn push(&mut self, node: Node<'a>) {
        self.children.push(node);
    }
}

/// Checks for the presence of a key in the context. Immediatly return failure if it doesn't.
#[derive(Debug)]
pub struct BlackboardNode<'a> {
    child: Box<Node<'a>>,
    key: String,
}

impl <'a> BehaviourTreeNode for BlackboardNode<'a> {
    fn visit(&mut self, context: &mut Context) -> VisitResult {
        if !context.map.contains_key::<str>(self.key.as_ref()) {
            return VisitResult::Failure;
        }
        self.child.visit(context)
    }
}

impl <'a> BlackboardNode<'a> {
    #[allow(dead_code)]
    // Kept just in case ...
    pub fn new(child: Box<Node<'a>>, key: String) -> BlackboardNode<'a> {
        BlackboardNode{child: child, key: key}
    }
}

/// Inverts the output of the child
#[derive(Debug)]
pub struct InverterNode<'a> {
    child: Box<Node<'a>>,
}

impl <'a> BehaviourTreeNode for InverterNode<'a> {
    fn visit(&mut self, context: &mut Context) -> VisitResult {
        match self.child.visit(context) {
            VisitResult::Success => return VisitResult::Failure,
            VisitResult::Failure => return VisitResult::Success,
            VisitResult::Running => return VisitResult::Running,
        }
    }
}

impl <'a> InverterNode<'a> {
    pub fn new(child: Box<Node<'a>>) -> InverterNode<'a> {
        InverterNode{child: child}
    }
}

pub enum Node<'a> {
    Leaf(LeafNode<'a>),
    Sequence(SequenceNode<'a>),
    Priority(PriorityNode<'a>),
    Selector(SelectorNode<'a>),
    Blackboard(BlackboardNode<'a>),
    Inverter(InverterNode<'a>),
}

#[allow(unused_must_use)]
impl <'a> Debug for Node<'a> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(),fmt::Error> {
        match *self {
            Node::Leaf(ref node) => {formatter.write_str("Leaf {"); node.fmt(formatter); formatter.write_str("}")}
            Node::Sequence(ref node) => {formatter.write_str("Sequence {"); node.fmt(formatter); formatter.write_str("}")}
            Node::Priority(ref node) => {formatter.write_str("Priority {"); node.fmt(formatter);formatter.write_str("}")}
            Node::Selector(ref node) => {formatter.write_str("Selector {"); node.fmt(formatter);formatter.write_str("}")}
            Node::Blackboard(ref node) => {formatter.write_str("Blackboard {"); node.fmt(formatter);formatter.write_str("}")}
            Node::Inverter(ref node) => {formatter.write_str("Inverter {"); node.fmt(formatter);formatter.write_str("}")}
        }
    }
}

impl <'a> BehaviourTreeNode for Node<'a> {
    fn visit(&mut self, context: &mut Context) -> VisitResult {
        match *self {
            Node::Leaf(ref mut node) => node.visit(context),
            Node::Sequence(ref mut node) => node.visit(context),
            Node::Priority(ref mut node) => node.visit(context),
            Node::Selector(ref mut node) => node.visit(context),
            Node::Blackboard(ref mut node) => node.visit(context),
            Node::Inverter(ref mut node) => node.visit(context),
        }
    }
}
