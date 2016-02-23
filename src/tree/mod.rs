mod non_optimized;
pub mod factory;

use std::collections::HashMap;
use std::fmt::{self,Debug,Formatter};

use flat_tree::FlatTree;
use flat_tree::buffer::ChildrenMut;

// To be replace by Lycan structures
struct StateSnapshot;
struct NotificationSummary;

#[derive(Debug)]
pub enum StoreKind {
    String(String),
    I64(i64),
}


pub struct Context {
    pub map: HashMap<String, StoreKind>,
    notif_summary: NotificationSummary,
    state: StateSnapshot,
}

impl Context {
    pub fn new(map: HashMap<String, StoreKind>) -> Context {
        Context {
            map: map,
            notif_summary: NotificationSummary,
            state: StateSnapshot,
        }
    }
}

#[derive(Debug,Copy,Eq,PartialEq,Clone)]
pub enum VisitResult {
    Success,
    Failure,
    Running,
}


pub trait BehaviourTreeNode {
    fn visit(&mut self, context: &mut Context) -> VisitResult;
}

impl <T> BehaviourTreeNode for T
where T: FnMut(&mut Context) -> VisitResult {
    fn visit(&mut self, context: &mut Context) -> VisitResult {
        self(context)
    }
}
/// Carries an action of checks a condition
///
/// Leaf nodes are the only nodes that actually do something in the game
pub struct LeafNode<'a> {
    inner: Box<BehaviourTreeNode + 'a>,
}

impl <'a> Debug for LeafNode<'a> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(),fmt::Error> {
        formatter.write_str("[leaf node]")
    }
}

impl <'a> LeafNode<'a> {
    pub fn new(child: Box<BehaviourTreeNode + 'a>) -> LeafNode<'a> {
        LeafNode{inner: child}
    }
}

impl <'a> BehaviourTreeNode for LeafNode<'a> {
    fn visit(&mut self, context: &mut Context) -> VisitResult {
        self.inner.visit(context)
    }
}

#[derive(Debug)]
pub struct OptimizedTree<'a> {
    inner: FlatTree<OptimizedNode<'a>>,
}

impl <'a> BehaviourTreeNode for OptimizedTree<'a> {
    fn visit(&mut self, context: &mut Context) -> VisitResult {
        let (root, children) = self.inner.tree_iter_mut()
                               .nth(0).expect("Tried to visit a tree without node");
        root.visit(context, children)
    }
}

#[derive(Debug)]
enum OptimizedNode<'a> {
    Leaf(OptimizedLeafNode<'a>),
    Sequence(OptimizedSequenceNode),
    Inverter,
    Priority,
    Blackboard(OptimizedBlackboardNode),
    Selector(OptimizedSelectorNode),
}

type OptimizedLeafNode<'a> = LeafNode<'a>;

#[derive(Debug)]
struct OptimizedSequenceNode {
    running: Option<usize>,
}

impl OptimizedSequenceNode {
    fn visit(&mut self, context: &mut Context, mut children: ChildrenMut<OptimizedNode>) -> VisitResult {
        let mut index = self.running.unwrap_or(0);
        let mut children = children.children_mut();

        // Go the the last previous running node
        for _ in 0..index {
            children.next();
        }
        for (child, grandchildren) in children {
            match child.visit(context, grandchildren) {
                VisitResult::Running => {
                    self.running = Some(index);
                    return VisitResult::Running;
                }
                VisitResult::Failure => {
                    return VisitResult::Failure;
                }
                VisitResult::Success => {}
            }
            index = index + 1;
        }
        VisitResult::Success
    }
}

#[derive(Debug)]
struct OptimizedSelectorNode {
    running: Option<usize>,
}

impl OptimizedSelectorNode {
    fn visit(&mut self, context: &mut Context, mut children: ChildrenMut<OptimizedNode>) -> VisitResult {
        let mut index = self.running.unwrap_or(0);
        let mut children = children.children_mut();

        // Go the the last previous running node
        for _ in 0..index {
            children.next();
        }
        for (child, grandchildren) in children {
            match child.visit(context, grandchildren) {
                VisitResult::Running => {
                    self.running = Some(index);
                    return VisitResult::Running;
                }
                VisitResult::Success => {
                    return VisitResult::Success;
                }
                VisitResult::Failure => {}
            }
            index = index + 1;
        }
        VisitResult::Failure
    }
}

#[derive(Debug)]
struct OptimizedBlackboardNode {
    key: String,
}

impl OptimizedBlackboardNode {
    fn visit(&mut self, context: &mut Context, mut children: ChildrenMut<OptimizedNode>) -> VisitResult {
        if !context.map.contains_key::<str>(self.key.as_ref()) {
            return VisitResult::Failure;
        }
        let (mut child, grandchildren) = children.get_mut(0).expect("Blackboard node without child");
        child.visit(context, grandchildren)
    }
}

impl <'a> OptimizedNode<'a> {
    fn visit(&mut self, context: &mut Context, children: ChildrenMut<OptimizedNode>) -> VisitResult {
        match *self {
            OptimizedNode::Sequence(ref mut node) => node.visit(context, children),
            OptimizedNode::Inverter => inverter_visit(context, children),
            OptimizedNode::Leaf(ref mut node) => node.visit(context),
            OptimizedNode::Priority => priority_visit(context, children),
            OptimizedNode::Selector(ref mut node) => node.visit(context, children),
            OptimizedNode::Blackboard(ref mut node) => node.visit(context, children),
        }
    }

    fn sequence(running: Option<usize>) -> OptimizedNode<'a> {
        OptimizedNode::Sequence(OptimizedSequenceNode{ running: running })
    }

    fn selector(running: Option<usize>) -> OptimizedNode<'a> {
        OptimizedNode::Selector(OptimizedSelectorNode{ running: running })
    }

    fn blackboard(key: String) -> OptimizedNode<'a> {
        OptimizedNode::Blackboard(OptimizedBlackboardNode{ key: key })
    }
}

fn inverter_visit(context: &mut Context, mut children: ChildrenMut<OptimizedNode>) -> VisitResult {
    let (child, grandchildren) = children.get_mut(0).expect("Inverter without children");
    match child.visit(context, grandchildren) {
        VisitResult::Success => VisitResult::Failure,
        VisitResult::Failure => VisitResult::Success,
        VisitResult::Running => VisitResult::Running,
    }
}

fn priority_visit(context: &mut Context, mut children: ChildrenMut<OptimizedNode>) -> VisitResult {
    let children = children.children_mut();
    for (child, grandchildren) in children {
        match child.visit(context, grandchildren) {
            VisitResult::Running => {
                return VisitResult::Running;
            }
            VisitResult::Failure => {
                return VisitResult::Failure;
            }
            VisitResult::Success => {}
        }
    }
    VisitResult::Success
}
