use crate::{evaluator::Environment, types::Node};

#[derive(PartialEq, Debug, Clone)]
pub enum Object {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Null,
    ReturnValue(Box<Object>),
    Function {
        parameters: Vec<Node>,
        body: Box<Node>,
        env: Box<Environment>,
    },
}

impl Object {
    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(value) => value.to_string(),
            Object::Float(value) => value.to_string(),
            Object::Boolean(value) => value.to_string(),
            Object::String(value) => value.clone(),
            Object::Null => "null".to_string(),
            Object::ReturnValue(value) => value.inspect(),
            Object::Function { parameters, body, env } => format!("fn({}){}", parameters.iter().map(|node| node.string()).collect::<String>(), body.string()),
        }
    }
}

pub const TRUE: Object = Object::Boolean(true);
pub const FALSE: Object = Object::Boolean(false);
pub const NULL: Object = Object::Null;