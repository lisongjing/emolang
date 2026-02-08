#[derive(PartialEq, PartialOrd, Debug)]
pub enum Object {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Null,
}

impl Object {
    pub fn to_bool(&self) -> bool {
        match self {
            Object::Integer(value) => *value > 0 ,
            Object::Float(value) => *value > 0.0,
            Object::Boolean(value) => *value,
            Object::String(value) => !value.is_empty(),
            Object::Null => false,
        }
    }
}

impl Object {
    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(value) => value.to_string(),
            Object::Float(value) => value.to_string(),
            Object::Boolean(value) => value.to_string(),
            Object::String(value) => value.clone(),
            Object::Null => "null".to_string(),
        }
    }
}

pub const TRUE: Object = Object::Boolean(true);
pub const FALSE: Object = Object::Boolean(false);
pub const NULL: Object = Object::Null;