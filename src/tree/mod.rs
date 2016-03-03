mod non_optimized;
pub mod factory;

use flat_tree::FlatTree;
use flat_tree::buffer::ChildrenMut;


#[derive(Debug,Copy,Eq,PartialEq,Clone)]
pub enum VisitResult {
    Success,
    Failure,
    Running,
}


pub trait BehaviourTreeNode<C> {
    fn visit(&mut self, context: &mut C) -> VisitResult;
}

pub struct Closure<T>(T);

impl <T,C> BehaviourTreeNode<C> for Closure<T>
where T: FnMut(&mut C) -> VisitResult {
    fn visit(&mut self, context: &mut C) -> VisitResult {
        self.0(context)
    }
}

impl <T: ?Sized, C> BehaviourTreeNode<C> for Box<T>
where T: BehaviourTreeNode<C> {
    fn visit(&mut self, context: &mut C) -> VisitResult {
        (**self).visit(context)
    }
}

pub trait LeafNodeFactory {
    type Output;
    fn instanciate(&self) -> Self::Output;
}

pub struct Prototype<T: Clone + BehaviourTreeNode<C>,C> {
    pub inner: T,
    _marker: ::std::marker::PhantomData<C>,
}

impl <T: Clone + BehaviourTreeNode<C>, C> Prototype<T,C> {
    pub fn new(inner: T) -> Prototype<T,C> {
        Prototype {
            inner: inner,
            _marker: ::std::marker::PhantomData,
        }
    }
}

impl <T,U> LeafNodeFactory for Closure<T>
where T: Fn() -> U {
    type Output = U;
    fn instanciate(&self) -> U {
        self.0()
    }
}

impl <T, C> LeafNodeFactory for Prototype<T,C>
where T: Clone,
      T: BehaviourTreeNode<C>,
      T: 'static {
    type Output = Box<BehaviourTreeNode<C>>;
    fn instanciate(&self) -> Self::Output {
        Box::new(self.inner.clone())
    }
}

impl <T: ?Sized> LeafNodeFactory for Box<T>
where T: LeafNodeFactory {
    type Output = T::Output;
    fn instanciate(&self) -> Self::Output {
        (**self).instanciate()
    }
}

/// Carries an action of checks a condition
///
/// Leaf nodes are the only nodes that actually do something in the game
#[derive(Debug,Clone)]
pub struct LeafNode<A> {
    inner: A,
}

impl <A> LeafNode<A> {
    pub fn new(inner: A) -> LeafNode<A> {
        LeafNode{inner: inner}
    }
}

impl <C, A> BehaviourTreeNode<C> for LeafNode<A>
where A: BehaviourTreeNode<C> {
    fn visit(&mut self, context: &mut C) -> VisitResult {
        self.inner.visit(context)
    }
}

#[derive(Debug)]
pub struct OptimizedTree<A> {
    inner: FlatTree<OptimizedNode<A>>,
}

impl <C,A> BehaviourTreeNode<C> for OptimizedTree<A>
where A: BehaviourTreeNode<C> {
    fn visit(&mut self, context: &mut C) -> VisitResult {
        let (root, children) = self.inner.tree_iter_mut()
                               .nth(0).expect("Tried to visit a tree without node");
        root.visit(context, children)
    }
}

#[derive(Debug)]
enum OptimizedNode<A> {
    Leaf(OptimizedLeafNode<A>),
    Sequence(OptimizedSequenceNode),
    Inverter,
    Priority,
    Selector(OptimizedSelectorNode),
}

type OptimizedLeafNode<A> = LeafNode<A>;

#[derive(Debug)]
struct OptimizedSequenceNode {
    running: Option<usize>,
}

impl OptimizedSequenceNode {
    fn visit<A,C>(&mut self, context: &mut C, mut children: ChildrenMut<OptimizedNode<A>>) -> VisitResult
    where A: BehaviourTreeNode<C> {
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
    fn visit<A,C>(&mut self, context: &mut C, mut children: ChildrenMut<OptimizedNode<A>>) -> VisitResult
    where A: BehaviourTreeNode<C> {
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

impl <A> OptimizedNode<A> {
    fn visit<C>(&mut self, context: &mut C, children: ChildrenMut<OptimizedNode<A>>) -> VisitResult
    where A: BehaviourTreeNode<C> {
        match *self {
            OptimizedNode::Sequence(ref mut node) => node.visit(context, children),
            OptimizedNode::Inverter => inverter_visit(context, children),
            OptimizedNode::Leaf(ref mut node) => node.visit(context),
            OptimizedNode::Priority => priority_visit(context, children),
            OptimizedNode::Selector(ref mut node) => node.visit(context, children),
        }
    }

    fn sequence(running: Option<usize>) -> OptimizedNode<A> {
        OptimizedNode::Sequence(OptimizedSequenceNode{ running: running })
    }

    fn selector(running: Option<usize>) -> OptimizedNode<A> {
        OptimizedNode::Selector(OptimizedSelectorNode{ running: running })
    }
}

fn inverter_visit<A,C>(context: &mut C, mut children: ChildrenMut<OptimizedNode<A>>) -> VisitResult
where A: BehaviourTreeNode<C> {
    let (child, grandchildren) = children.get_mut(0).expect("Inverter without children");
    match child.visit(context, grandchildren) {
        VisitResult::Success => VisitResult::Failure,
        VisitResult::Failure => VisitResult::Success,
        VisitResult::Running => VisitResult::Running,
    }
}

fn priority_visit<A,C>(context: &mut C, mut children: ChildrenMut<OptimizedNode<A>>) -> VisitResult
where A: BehaviourTreeNode<C> {
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
