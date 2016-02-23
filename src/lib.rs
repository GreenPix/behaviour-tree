#![feature(plugin)]
#![plugin(phf_macros)]
extern crate phf;
extern crate flat_tree;
extern crate ref_slice;

pub use parser::parse;
mod parser;
pub mod tree;
pub mod standard;