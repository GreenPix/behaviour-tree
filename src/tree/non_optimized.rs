use std::fmt::{self,Debug,Formatter};

use super::{VisitResult,BehaviourTreeNode,LeafNode};

#[derive(Debug)]
pub struct Tree<A> {
    root: Node<A>,
}

impl <A> Tree<A> {
    pub fn new(root: Node<A>) -> Tree<A> {
        Tree {
            root: root,
        }
    }
}

impl <A,C> BehaviourTreeNode<C> for Tree<A>
where A: BehaviourTreeNode<C> {
    fn visit(&mut self, context: &mut C) -> VisitResult {
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
pub struct SequenceNode<A> {
    running: Option<usize>,
    children: Vec<Node<A>>,
}

impl <A,C> BehaviourTreeNode<C> for SequenceNode<A>
where A: BehaviourTreeNode<C> {
    fn visit(&mut self, context: &mut C) -> VisitResult {
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

impl <A> SequenceNode<A> {
    pub fn new(children: Vec<Node<A>>) -> SequenceNode<A> {
        SequenceNode {
            running: None,
            children: children,
        }
    }

    #[allow(dead_code)]
    // Kept just in case ...
    pub fn push(&mut self, node: Node<A>) {
        self.children.push(node);
    }
}

/// Counterpart of Sequence: returns Success on the first child returning Success, and return
/// Failure if all children fail.
///
/// This is typically used when a set of actions have the same objective, but those actions are
/// classified by preference.
#[derive(Debug)]
pub struct SelectorNode<A> {
    running: Option<usize>,
    children: Vec<Node<A>>,
}

impl <A,C> BehaviourTreeNode<C> for SelectorNode<A>
where A: BehaviourTreeNode<C> {
    fn visit(&mut self, context: &mut C) -> VisitResult {
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

impl <A> SelectorNode<A> {
    pub fn new(children: Vec<Node<A>>) -> SelectorNode<A> {
        SelectorNode {
            running: None,
            children: children,
        }
    }

    #[allow(dead_code)]
    // Kept just in case ...
    pub fn push(&mut self, node: Node<A>) {
        self.children.push(node);
    }
}

/// Same as Sequence, but do not remember the last running child and revisit all children
#[derive(Debug)]
pub struct PriorityNode<A> {
    children: Vec<Node<A>>,
}

impl <A,C> BehaviourTreeNode<C> for PriorityNode<A>
where A: BehaviourTreeNode<C> {
    fn visit(&mut self, context: &mut C) -> VisitResult {
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

impl <A> PriorityNode<A> {
    pub fn new(children: Vec<Node<A>>) -> PriorityNode<A> {
        PriorityNode{children: children}
    }

    #[allow(dead_code)]
    // Kept just in case ...
    pub fn push(&mut self, node: Node<A>) {
        self.children.push(node);
    }
}

/// Inverts the output of the child
#[derive(Debug)]
pub struct InverterNode<A> {
    child: Box<Node<A>>,
}

impl <A,C> BehaviourTreeNode<C> for InverterNode<A>
where A: BehaviourTreeNode<C> {
    fn visit(&mut self, context: &mut C) -> VisitResult {
        match self.child.visit(context) {
            VisitResult::Success => return VisitResult::Failure,
            VisitResult::Failure => return VisitResult::Success,
            VisitResult::Running => return VisitResult::Running,
        }
    }
}

impl <A> InverterNode<A> {
    pub fn new(child: Box<Node<A>>) -> InverterNode<A> {
        InverterNode{child: child}
    }
}

pub enum Node<A> {
    Leaf(LeafNode<A>),
    Sequence(SequenceNode<A>),
    Priority(PriorityNode<A>),
    Selector(SelectorNode<A>),
    Inverter(InverterNode<A>),
}

impl <A> Debug for Node<A> {
    fn fmt(&self, _f: &mut Formatter) -> Result<(),fmt::Error> {
        // TODO: Derive does not work any more because of the type parameter A
        /*
        match *self {
            Node::Leaf(ref node) => {formatter.write_str("Leaf {"); node.fmt(formatter); formatter.write_str("}")}
            Node::Sequence(ref node) => {formatter.write_str("Sequence {"); node.fmt(formatter); formatter.write_str("}")}
            Node::Priority(ref node) => {formatter.write_str("Priority {"); node.fmt(formatter);formatter.write_str("}")}
            Node::Selector(ref node) => {formatter.write_str("Selector {"); node.fmt(formatter);formatter.write_str("}")}
            Node::Blackboard(ref node) => {formatter.write_str("Blackboard {"); node.fmt(formatter);formatter.write_str("}")}
            Node::Inverter(ref node) => {formatter.write_str("Inverter {"); node.fmt(formatter);formatter.write_str("}")}
        }
        */
        Ok(())
    }
}

impl <A,C> BehaviourTreeNode<C> for Node<A>
where A: BehaviourTreeNode<C> {
    fn visit(&mut self, context: &mut C) -> VisitResult {
        match *self {
            Node::Leaf(ref mut node) => node.visit(context),
            Node::Sequence(ref mut node) => node.visit(context),
            Node::Priority(ref mut node) => node.visit(context),
            Node::Selector(ref mut node) => node.visit(context),
            Node::Inverter(ref mut node) => node.visit(context),
        }
    }
}
