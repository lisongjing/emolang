use std::{collections::HashMap, sync::Arc};

use crate::{types::Node, util::emoji_convert::{null_emoji, boolean_to_emoji, float_to_emoji, integer_to_emoji}};

#[derive(PartialEq, Debug, Clone)]
pub enum Object {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Null,
    List(Vec<Object>),
    ReturnValue(Box<Object>),
    Function {
        parameters: Vec<Node>,
        body: Box<Node>,
        env: Box<Environment>,
    },
    BuiltinFunction(BuiltinFunction),
}

impl Object {
    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(value) => value.to_string(),
            Object::Float(value) => value.to_string(),
            Object::Boolean(value) => value.to_string(),
            Object::String(value) => value.clone(),
            Object::Null => "null".to_string(),
            Object::List(value) => format!("[{}]", value.iter().map(|obj| obj.inspect()).collect::<Vec<String>>().join(", ")),
            Object::ReturnValue(value) => value.inspect(),
            Object::Function { parameters, body, env: _ } => format!("fn({}) {}", parameters.iter().map(|node| node.string()).collect::<Vec<String>>().join(", "), body.string()),
            Object::BuiltinFunction(function) => format!("{}(args...){{ //builtin implementation }}", function.name()),
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
        let mut map = HashMap::new();

        BuiltinFunction::register(&mut map);

        Environment { map, outer: None }
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

type FunctionWrapper = Arc<dyn Fn(&[Object]) -> Result<Object, String> + Send + Sync>;

#[derive(PartialEq, Debug, Clone)]
pub enum BuiltinFunction {
    ToString,
    Print,
    Println,
}

impl BuiltinFunction {

    const ALL: [BuiltinFunction;3] = [BuiltinFunction::ToString, BuiltinFunction::Print, BuiltinFunction::Println];

    pub fn name(&self) -> String {
        match self {
            BuiltinFunction::ToString => String::from("👁️‍🗨️"),
            BuiltinFunction::Print => String::from("🖨️"),
            BuiltinFunction::Println => String::from("🖨️↩️"),
        }
    }

    pub fn function(&self) -> FunctionWrapper {
        match self {
            BuiltinFunction::ToString => Arc::new(BuiltinFunction::to_string) as FunctionWrapper,
            BuiltinFunction::Print => Arc::new(BuiltinFunction::print) as FunctionWrapper,
            BuiltinFunction::Println => Arc::new(BuiltinFunction::println) as FunctionWrapper,
        }
    }

    // api

    pub fn register(map: &mut HashMap<String, Object>) {
        for function in BuiltinFunction::ALL {
            map.insert(function.name(), Object::BuiltinFunction(function));
        }
    }

    pub fn call(&self, args: &[Object]) -> Result<Object, String> {
        self.function()(args)
    }

    // builtin function implementations

    fn to_string(args: &[Object]) -> Result<Object, String> {
        if args.len() != 1 {
            return Err(format!("Expected 1 argument(s), but got {}", args.len()));
        }

        let string = match args.first().unwrap() {
            Object::String(value) => value.clone(),
            Object::Integer(value) => integer_to_emoji(value),
            Object::Float(value) => float_to_emoji(value),
            Object::Boolean(value) => boolean_to_emoji(value),
            Object::Null => null_emoji(),
            _ => return Err(format!("Incompatible argument type with string: {:?}", args.first().unwrap())),
        };

        Ok(Object::String(string))
    }

    fn print(args: &[Object]) -> Result<Object, String> {
        if let Object::String(string) = BuiltinFunction::to_string(args)? {
            print!("{string}");
        }
        Ok(Object::Null)
    }

    fn println(args: &[Object]) -> Result<Object, String> {
        if let Object::String(string) = BuiltinFunction::to_string(args)? {
            println!("{string}");
        }
        Ok(Object::Null)
    }
}
