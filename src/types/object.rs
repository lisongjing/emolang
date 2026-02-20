use std::collections::HashMap;

use crate::types::Node;

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
            Object::Function { parameters, body, env: _ } => format!("fn({}){}", parameters.iter().map(|node| node.string()).collect::<String>(), body.string()),
        }
    }
}

pub const TRUE: Object = Object::Boolean(true);
pub const FALSE: Object = Object::Boolean(false);
pub const NULL: Object = Object::Null;


#[derive(PartialEq, Debug, Clone)]
pub struct Environment {
    map: HashMap<String, Object>,
    outer: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment { map: HashMap::new(), outer: None }
    }

    pub fn new_enclosed(outer: Box<Environment>) -> Self {
        Environment { map: HashMap::new(), outer: Some(outer) }
    }

    pub fn set(&mut self, identifier: String, value: Object) {
        self.map.insert(identifier, value);
    }

    pub fn get(&self, identifier: &String) -> Option<&Object> {
        let mut obj = self.map.get(identifier);
        if obj.is_none() && let Some(outer) = &self.outer {
            obj = outer.get(identifier);
        }
        obj
    }
}