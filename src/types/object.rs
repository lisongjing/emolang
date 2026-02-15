#[derive(PartialEq, PartialOrd, Debug)]
pub enum Object {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Null,
    ReturnValue(Box<Object>),
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
        }
    }
}

pub const TRUE: Object = Object::Boolean(true);
pub const FALSE: Object = Object::Boolean(false);
pub const NULL: Object = Object::Null;