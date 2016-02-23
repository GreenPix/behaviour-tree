use tree::{LeafNode,Context,VisitResult};
use standard::{Value,LeafNodeFactory};

pub fn always_running(_: &Option<Value>) -> Result<LeafNodeFactory, String> {
    Ok(Box::new(move || LeafNode::new(Box::new(move |_: &mut Context| {
        VisitResult::Running
    }))))
}
