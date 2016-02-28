extern crate flat_tree;
extern crate ref_slice;

pub use parser::parse;
pub use self::tree::OptimizedTree as BehaviourTree;
mod parser;
pub mod tree;
pub mod standard;
