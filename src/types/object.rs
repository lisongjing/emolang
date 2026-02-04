pub enum ObjectType {
    Integer,
    Float,
    Boolean,
    String,
    Null,
}

pub trait Object {
    fn object_type(&self) -> ObjectType;
    fn inspect(&self) -> String;
}

pub struct Integer {
    value: i64,
}

impl Object for Integer {
    fn object_type(&self) -> ObjectType {
        ObjectType::Integer
    }

    fn inspect(&self) -> String {
        self.value.to_string()
    }
}

pub struct Float {
    value: f64,
}

impl Object for Float {
    fn object_type(&self) -> ObjectType {
        ObjectType::Float
    }

    fn inspect(&self) -> String {
        self.value.to_string()
    }
}

pub struct Boolean {
    value: bool,
}

impl Object for Boolean {
    fn object_type(&self) -> ObjectType {
        ObjectType::Boolean
    }

    fn inspect(&self) -> String {
        self.value.to_string()
    }
}

pub struct MutableString {
    value: String,
}

impl Object for MutableString {
    fn object_type(&self) -> ObjectType {
        ObjectType::String
    }

    fn inspect(&self) -> String {
        self.value.clone()
    }
}

pub struct EmoNull {}

impl Object for EmoNull {
    fn object_type(&self) -> ObjectType {
        ObjectType::Null
    }

    fn inspect(&self) -> String {
        "null".to_string()
    }
}