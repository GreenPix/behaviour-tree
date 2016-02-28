use std::collections::HashMap;
use std::fmt::{self,Debug,Formatter};
use std::hash::{Hash,BuildHasher};
use std::borrow::Borrow;

use tree::{LeafNode,VisitResult,BehaviourTreeNode,Prototype};
use tree::{LeafNodeFactory};
use parser::FactoryProducer;

//mod fake_nodes;
//pub mod expressions;
//mod conditions;

pub type StandardFactory<C> = Box<LeafNodeFactory<Output=Box<BehaviourTreeNode<C>>>>;
pub trait LeafNodeFactoryFactory {
    type Output;
    fn create_factory(&self, options: &Option<Value>) -> Result<Self::Output,String>;
}

impl <T,U> LeafNodeFactoryFactory for T
where T: Fn(&Option<Value>) -> Result<U,String> {
    type Output = U;
    fn create_factory(&self, option: &Option<Value>) -> Result<Self::Output,String> {
        self(option)
    }
}

impl <C: 'static> FactoryProducer for LeavesCollection<C> {
    type Factory = StandardFactory<C>;
    fn generate_leaf(&self, name: &str, option: &Option<Value>) -> Result<Self::Factory,String> {
        match self.inner.get(name) {
            None => Err(format!("Could not find leaf with name {}", name)),
            Some(fact_fact) => {
                let fact = try!(fact_fact.create_factory(option));
                Ok(fact) 
            }
        }
    }
}

/// A trait to abstract an object that can be queried with a key and will return a value
pub trait Gettable<K: ?Sized,V: ?Sized> {
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

impl <'a, T: ?Sized ,Q: ?Sized,V> Gettable<Q,V> for &'a T
where T: Gettable<Q,V> {
    fn get(&self, k: &Q) -> Option<&V> {
        (*self).get(k)
    }
}

// XXX: Context stuff is unclear, we want to avoid useless string allocation/copy
pub trait Context: Gettable<str,StoreKind> {
    fn insert_value(&mut self, key: String, value: StoreKind);
    fn set_value(&mut self, key: &str, value: StoreKind) -> Result<(),()>;
}

impl <S: BuildHasher> Context for HashMap<String,StoreKind,S> {
    fn insert_value(&mut self, key: String, value: StoreKind) {
        self.insert(key,value);
    }

    fn set_value(&mut self, key: &str, value: StoreKind) -> Result<(),()> {
        match self.get_mut(key) {
            Some(v) => {
                *v = value;
                Ok(())
            }
            None => Err(()),
        }
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

#[derive(Debug,Clone)]
pub struct PrintText {
    pub text: String,
}

impl <C> BehaviourTreeNode<C> for PrintText {
    fn visit(&mut self, _context: &mut C) -> VisitResult {
        println!("Message node: {}", self.text);
        VisitResult::Success
    }
}

pub fn print_text<C: 'static>(options: &Option<Value>) -> Result<StandardFactory<C>, String> {
    let message_orig = match options {
        &Some(Value::String(ref message)) => message,
        other => return Err(format!("Expected message, found {:?}", other)),
    };

    let message = message_orig.replace("_"," ");

    Ok(Box::new(Prototype::new(PrintText { text: message })))
}

/*
TODO: Finish this

#[derive(Debug,Clone)]
pub struct Increment {
    pub variable: String,
    pub value: i64,
}

impl <C: Context> BehaviourTreeNode<C> for Increment {
    fn visit(&mut self, context: &mut C) -> VisitResult {
        let current_value = match context.get(&self.variable) {
            None => {
                None
            },
            Some(&StoreKind::I64(variable)) => {
                Some(variable)
            },
            Some(other) => {
                println!("Expected integer variable for key {}, found {:?}", variable_name, other);
                return VisitResult::Failure
            }
        };
        match current_value {
            Some(v) => {
                match context.set_value(&self.variable, v + self.value) {
                    Ok(_) => VisitResult::Success,
                    Err(_) => {
                        warning!("Context::set_value failed for variable {} after a successfull get", self.variable);
                        VisitResult::Success,
                    }
                }
            }
            TODO
            None => context.insert_value(self.variable.clone(), self.value),
        }
        VisitResult::Success
    }
}

pub fn increment<C: Gettable<str,Value>>(options: &Option<Value>) -> Result<Box<LeafNodeFactory<C>>, String> {
    let options_map = match options {
        &Some(Value::Map(ref map)) => map,
        other => return Err(format!("Expected hashmap, found {:?}", other)),
    };
    let variable = match options_map.get("variable") {
        None => return Err(format!("Increment: missing required \"variable\" field")),
        Some(Value::String(ref name)) => name.clone(),
        Some(other) => return Err(format!("Increment: expected string for field \"variable\", got {:?}", other)),
    };
    let value = match options_map.get("value") {
        None => return Err(format!("Increment: missing required \"value\" field")),
        Some(Value::Integer(value)) => value,
        Some(other) => return Err(format!("Increment: expected integer for field \"value\", got {:?}", other)),
    };
    let increment = Increment {
        variable: variable,
        value: value,
    };
    Ok(Box::new(Prototype(increment)))
}
*/


#[derive(Default)]
pub struct LeavesCollection<C> {
    inner: HashMap<String,Box<LeafNodeFactoryFactory<Output=StandardFactory<C>>>>,
}

impl <C> LeavesCollection<C> {
    pub fn new() -> LeavesCollection<C> {
        LeavesCollection {
            inner: HashMap::new(),
        }
    }

    pub fn register_function(
        &mut self,
        key: String,
        f: Box<LeafNodeFactoryFactory<Output=StandardFactory<C>>>,
        ) {
        self.inner.insert(key,f);
    }
}

macro_rules! insert_all {
    ($($name:expr => $fun:expr),*) => (
        {
            let mut collection = LeavesCollection::new();
            $(
            collection.inner.insert(
                String::from($name),
                Box::new($fun),
                );
            )*
            collection
        }
        );
    ($($name:expr => $fun:expr),+,) => (
        insert_all!($($name => $fun),+)
        );
}
impl <C: Context + 'static> LeavesCollection<C> {
    pub fn standard() -> LeavesCollection<C> {
        let collection = insert_all!(
            "print_text" => print_text,
            //"increment" => increment,

            );

        collection
    }
}

#[derive(Debug)]
pub enum StoreKind {
    String(String),
    I64(i64),
}
