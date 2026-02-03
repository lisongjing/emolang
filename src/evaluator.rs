use std::any::{Any, TypeId};

use crate::parser::{IntegerLiteral, Node};

pub enum ObjectType {
    Integer,
    Float,
    Boolean,
    String,
    Null,
}

pub trait EmoObject {
    fn object_type(&self) -> ObjectType;
    fn inspect(&self) -> String;
}

pub struct EmoInteger {
    value: i64,
}

impl EmoObject for EmoInteger {
    fn object_type(&self) -> ObjectType {
        ObjectType::Integer
    }

    fn inspect(&self) -> String {
        self.value.to_string()
    }
}

pub struct EmoFloat {
    value: f64,
}

impl EmoObject for EmoFloat {
    fn object_type(&self) -> ObjectType {
        ObjectType::Float
    }

    fn inspect(&self) -> String {
        self.value.to_string()
    }
}

pub struct EmoBoolean {
    value: bool,
}

impl EmoObject for EmoBoolean {
    fn object_type(&self) -> ObjectType {
        ObjectType::Boolean
    }

    fn inspect(&self) -> String {
        self.value.to_string()
    }
}

pub struct EmoString {
    value: String,
}

impl EmoObject for EmoString {
    fn object_type(&self) -> ObjectType {
        ObjectType::String
    }

    fn inspect(&self) -> String {
        self.value.clone()
    }
}

pub struct EmoNull {}

impl EmoObject for EmoNull {
    fn object_type(&self) -> ObjectType {
        ObjectType::Null
    }

    fn inspect(&self) -> String {
        "null".to_string()
    }
}

pub fn eval(node: Box<dyn Node>) -> Box<dyn EmoObject> {
    Box::new(EmoNull {})
}