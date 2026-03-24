use std::{collections::HashMap, hash::Hash, sync::Arc};

use ordered_float::OrderedFloat;

use crate::{types::Node, util::emoji_convert::object_to_emoji};

#[derive(PartialEq, Debug, Clone)]
pub enum Object {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Null,
    List(Vec<Object>),
    Map(HashMap<Object, Object>),
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
            Object::String(value) => format!("\"{}\"", value),
            Object::Null => "null".to_string(),
            Object::List(value) => format!("[{}]", value.iter().map(|obj| obj.inspect()).collect::<Vec<String>>().join(", ")),
            Object::Map(value) => format!("{{{}}}", value.iter().map(|(k, v)| format!("{}: {}", k.inspect(), v.inspect())).collect::<Vec<String>>().join(", ")),
            Object::ReturnValue(value) => value.inspect(),
            Object::Function { parameters, body, env: _ } => format!("fn({}) {}", parameters.iter().map(|node| node.string()).collect::<Vec<String>>().join(", "), body.string()),
            Object::BuiltinFunction(function) => format!("{}(args...){{ //builtin implementation }}", function.name()),
        }
    }
}

impl Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {

        match self {
            Object::Integer(value) => {
                0u32.hash(state);
                value.hash(state);
            },
            Object::Float(value) => {
                1u32.hash(state);
                OrderedFloat(*value).hash(state);
            },
            Object::Boolean(value) => {
                2u32.hash(state);
                value.hash(state);
            },
            Object::String(value) => {
                3u32.hash(state);
                value.hash(state);
            },
            Object::Null => {
                4u32.hash(state);
            },
            Object::List(elements) => {
                5u32.hash(state);
                for element in elements {
                    element.hash(state);
                }
            },
            Object::Map(entries) => {
                6u32.hash(state);
                for (key, value) in entries {
                    key.hash(state);
                    value.hash(state);
                }
            },
            Object::ReturnValue(value) => {
                7u32.hash(state);
                value.hash(state);
            },
            Object::Function { parameters: _, body, env: _ } => {
                8u32.hash(state);
                body.string().hash(state);
            },
            Object::BuiltinFunction(value) => {
                9u32.hash(state);
                value.name().hash(state);
            },
        }
    }
}

impl Eq for Object {
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

        object_to_emoji(args.first().unwrap())
            .map(Object::String)
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
