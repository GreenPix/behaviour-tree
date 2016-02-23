use std::collections::HashMap;
use std::fmt::{self,Debug,Formatter};
use std::hash::{Hash,BuildHasher};
use std::borrow::Borrow;

use phf::{self,PhfHash};

use tree::{LeafNode,Context,VisitResult,StoreKind};

pub type LeafNodeFactory = Box<Fn() -> LeafNode<'static>>;
/// Type of the functions the user must provide
///
/// They are supposed to take options as parameter, and create an object used to instanciate leaf
/// node for actual trees
pub type LeafNodeFactoryFactory = fn(&Option<Value>) -> Result<LeafNodeFactory, String>;

mod fake_nodes;
pub mod expressions;
mod conditions;

/// A trait to abstract an object that can be queried with a key and will return a value
pub trait Gettable<K: ?Sized,V> {
    fn get(&self, k: &K) -> Option<&V>;
}

impl <K,V,S,Q: ?Sized> Gettable<Q,V> for HashMap<K,V,S>
where K: Hash + Eq,
      K: Borrow<Q>,
      Q: Hash + Eq,
      S: BuildHasher {
    fn get(&self, k: &Q) -> Option<&V> {
        self.get(k)
    }
}

impl <K,V,Q: ?Sized> Gettable<Q,V> for phf::Map<K,V>
where Q: Eq + PhfHash,
      K: Borrow<Q> {
    fn get(&self, k: &Q) -> Option<&V> {
        self.get(k)
    }
}

impl <T,Q: ?Sized,V> Gettable<Q,V> for [T]
where T: Gettable<Q,V> {
    fn get(&self, k: &Q) -> Option<&V> {
        for i in self.iter() {
            let value = i.get(k);
            if value.is_some() {
                return value;
            }
        }
        None
    }
}

impl <'a, T,Q: ?Sized,V> Gettable<Q,V> for &'a T
where T: Gettable<Q,V> {
    fn get(&self, k: &Q) -> Option<&V> {
        (*self).get(k)
    }
}

#[derive(Debug,Clone)]
pub enum Value {
    String(String),
    Map(HashMap<String,Value>),
    Array(Vec<Value>),
    Integer(i64),
    Operator(Operator),
    Unknown(char),
}

#[derive(Debug,Clone,Copy)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
}

#[allow(trivial_casts)]
pub static LEAVES_COLLECTION: phf::Map<&'static str, LeafNodeFactoryFactory> = phf_map! {
    "print_word" => self::print_word as LeafNodeFactoryFactory,
    "increment" => self::increment as LeafNodeFactoryFactory,
    "always_running" => fake_nodes::always_running as LeafNodeFactoryFactory,
    "evaluate_int" => expressions::evaluate_int_node as LeafNodeFactoryFactory,
    "check_condition" => conditions::check_condition_node as LeafNodeFactoryFactory,
};

impl Debug for LeafNodeFactory {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(),fmt::Error> {
        formatter.write_str("[leaf node factory]")
    }
}

fn print_word(options: &Option<Value>) -> Result<LeafNodeFactory, String> {
    let message_orig = match options {
        &Some(Value::String(ref message)) => message,
        other => return Err(format!("Expected message, found {:?}", other)),
    };

    let message = message_orig.replace("_"," ");

    Ok(Box::new(move || {
        let message = message.clone();
        let closure = Box::new(move |_: &mut Context| {
            println!("Message node: {}", message);
            VisitResult::Success
        });
        LeafNode::new(closure)
    }))
}

fn increment(options: &Option<Value>) -> Result<LeafNodeFactory, String> {
    let options_map = match options {
        &Some(Value::Map(ref map)) => map,
        other => return Err(format!("Expected hashmap, found {:?}", other)),
    };
    let list_key_values: Vec<(String,i64)> = options_map.iter().filter_map(|(key, value)| {
        match *value {
            Value::Integer(value) => Some((key.clone(),value)),
            _ => {
                println!("Warning: value was not an integer, ignoring entry {}", key);
                None
            }
        }
    }).collect();
    Ok(Box::new(move || {
        let list_key_values = list_key_values.clone();
        let closure = Box::new(move |context: &mut Context| {
            for &(ref variable_name, ref value) in list_key_values.iter() {
                match context.map.get_mut::<str>(variable_name.as_ref()) {
                    None => {},
                    Some(&mut StoreKind::I64(ref mut variable)) => {
                        *variable = *variable + value;
                        println!("Value of variable {} : {}",variable_name, *variable);
                        return VisitResult::Success;
                    },
                    Some(other) => {
                        println!("Expected integer variable for key {}, found {:?}", variable_name, other);
                        return VisitResult::Failure
                    }
                }
                context.map.insert(variable_name.clone(), StoreKind::I64(*value));
                println!("Variable {} did not exist, and is now equal to {}",variable_name, value);
            }
            VisitResult::Success
        });
        LeafNode::new(closure)
    }))
}
