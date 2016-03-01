use standard::{Value};
use tree::factory::{TreeFactory,NodeFactory};
use self::ast::Node;

mod parser;
mod ast;
mod lexer;

pub use self::lexer::{Token,Tokenizer};

pub trait FactoryProducer {
    type Factory;
    fn generate_leaf(&self, name: &str, option: &Option<Value>) -> Result<Self::Factory,String>;
}

pub fn parse<T: ?Sized>(
    input: &str,
    leaves: &T,
    ) -> Result<Vec<TreeFactory<T::Factory>>,String>
where T: FactoryProducer {
    let tokenizer = Tokenizer::new(input);
    let tokenizer_mapped = tokenizer.map(|e| {
        e.map(|token| ((),token,()))
    });
    let trees = match parser::parse_TreeCollection(tokenizer_mapped) {
        Ok(t) => t,
        Err(e) => {
            println!("Error: {:#?}", e);
            return Err(format!("Parsing error {:#?}", e));
        }
    };
    let mut new_trees = Vec::new();
    for tree in trees {
        let new_root = try!(resolve_dependencies(tree.root, leaves));
        let new_tree = TreeFactory::new(new_root, tree.name);
        new_trees.push(new_tree);
    }
    Ok(new_trees)
}

fn resolve_dependencies<T: ?Sized>(node: Node, leaves: &T) -> Result<NodeFactory<T::Factory>,String>
where T: FactoryProducer {
    match node {
        Node::Sequence(children) => {
            let new_children = try!(resolve_dependencies_vec(children, leaves));
            Ok(NodeFactory::new_sequence(new_children))
        }
        Node::Selector(children) => {
            let new_children = try!(resolve_dependencies_vec(children, leaves));
            Ok(NodeFactory::new_selector(new_children))
        }
        Node::Priority(children) => {
            let new_children = try!(resolve_dependencies_vec(children, leaves));
            Ok(NodeFactory::new_priority(new_children))
        }
        Node::Inverter(child) => {
            let new_child = try!(resolve_dependencies(*child,leaves));
            Ok(NodeFactory::new_inverter(Box::new(new_child)))
        }
        Node::Leaf(name, options) => {
            match leaves.generate_leaf(&name, &options) {
                Err(e) => Err(format!("Could not find leaf node {}: {}", name, e)),
                Ok(f) => {
                    Ok(NodeFactory::new_leaf(f))
                }
            }
        }
    }
}

fn resolve_dependencies_vec<T: ?Sized>(nodes: Vec<Node>, leaves: &T)
-> Result<Vec<NodeFactory<T::Factory>>, String>
where T: FactoryProducer {
    let mut new_nodes = Vec::new();
    for node in nodes {
        let new_node = try!(resolve_dependencies(node, leaves));
        new_nodes.push(new_node);
    }
    Ok(new_nodes)
}
