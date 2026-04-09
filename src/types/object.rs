use std::{
    collections::HashMap,
    hash::Hash,
    sync::Arc,
};

use ordered_float::OrderedFloat;
use unicode_segmentation::UnicodeSegmentation;

use crate::{types::Node, util::emoji_convert::object_to_emoji};

#[derive(PartialEq, Debug, Clone)]
pub enum Object {
    Integer(i64, Environment),
    Float(f64, Environment),
    Boolean(bool),
    String(String, Environment),
    Null,
    List(Vec<Object>, Environment),
    Map(HashMap<Object, Object>, Environment),
    ReturnValue(Box<Object>),
    Function {
        parameters: Vec<Node>,
        body: Box<Node>,
        env: Box<Environment>,
    },
    BuiltinFunction(BuiltinFunction),
}

impl Object {
    pub fn integer(value: i64) -> Object {
        Object::Integer(value, Environment::new_builtins(&[BuiltinFunction::Pow]))
    }

    pub fn float(value: f64) -> Object {
        Object::Float(value, Environment::new_builtins(&[BuiltinFunction::Pow]))
    }

    pub fn boolean(value: bool) -> Object {
        if value {TRUE} else {FALSE}
    }

    pub fn string(value: String) -> Object {
        Object::String(value, Environment::new_builtins(&[BuiltinFunction::Len]))
    }

    pub fn null() -> Object {
        NULL
    }

    pub fn list(value: Vec<Object>) -> Object {
        Object::List(value, Environment::new_builtins(&[BuiltinFunction::Len]))
    }

    pub fn map(value: HashMap<Object, Object>) -> Object {
        Object::Map(value, Environment::new_builtins(&[BuiltinFunction::Len]))
    }


    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(value, _) => value.to_string(),
            Object::Float(value, _) => value.to_string(),
            Object::Boolean(value) => value.to_string(),
            Object::String(value, _) => format!("\"{}\"", value),
            Object::Null => "null".to_string(),
            Object::List(value, _) => format!(
                "[{}]",
                value
                    .iter()
                    .map(|obj| obj.inspect())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Object::Map(value, _) => format!(
                "{{{}}}",
                value
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.inspect(), v.inspect()))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Object::ReturnValue(value) => value.inspect(),
            Object::Function {
                parameters,
                body,
                env: _,
            } => format!(
                "fn({}) {}",
                parameters
                    .iter()
                    .map(|node| node.string())
                    .collect::<Vec<String>>()
                    .join(", "),
                body.string()
            ),
            Object::BuiltinFunction(function) => {
                format!("{}(args...){{ //builtin implementation }}", function.name())
            }
        }
    }

    pub fn associated_env(&self) -> Environment {
        let mut env = match self {
            Object::Integer(_, env) => env.clone(),
            Object::Float(_, env) => env.clone(),
            Object::String(_, env) => env.clone(),
            Object::List(_, env) => env.clone(),
            Object::Map(_, env) => env.clone(),
            _ => Environment::new_builtins(&[]),
        };
        env.set("🈯".to_string(), self.clone());
        env
    }

    pub fn set_associated_env(&mut self, env: Environment) {
        match self {
            Object::Integer(_, e) => *e = env,
            Object::Float(_, e) => *e = env,
            Object::String(_, e) => *e = env,
            Object::List(_, e) => *e = env,
            Object::Map(_, e) => *e = env,
            _ => (),
        }
    }
}

impl Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Object::Integer(value, _) => {
                0u32.hash(state);
                value.hash(state);
            }
            Object::Float(value, _) => {
                1u32.hash(state);
                OrderedFloat(*value).hash(state);
            }
            Object::Boolean(value) => {
                2u32.hash(state);
                value.hash(state);
            }
            Object::String(value, _) => {
                3u32.hash(state);
                value.hash(state);
            }
            Object::Null => {
                4u32.hash(state);
            }
            Object::List(elements, _) => {
                5u32.hash(state);
                for element in elements {
                    element.hash(state);
                }
            }
            Object::Map(entries, _) => {
                6u32.hash(state);
                for (key, value) in entries {
                    key.hash(state);
                    value.hash(state);
                }
            }
            Object::ReturnValue(value) => {
                7u32.hash(state);
                value.hash(state);
            }
            Object::Function {
                parameters: _,
                body,
                env: _,
            } => {
                8u32.hash(state);
                body.string().hash(state);
            }
            Object::BuiltinFunction(value) => {
                9u32.hash(state);
                value.name().hash(state);
            }
        }
    }
}

impl Eq for Object {}

pub const TRUE: Object = Object::Boolean(true);
pub const FALSE: Object = Object::Boolean(false);
pub const NULL: Object = Object::Null;


#[derive(PartialEq, Debug, Clone)]
pub struct Environment {
    map: HashMap<String, Object>,
    outer: Option<Box<Environment>>,
}

impl Environment {
    pub fn new_default() -> Self {
        let mut map = HashMap::new();

        BuiltinFunction::register_exports(&mut map);

        Environment { map, outer: None }
    }

    pub fn new_enclosed(outer: Box<Environment>) -> Self {
        Environment {
            map: HashMap::new(),
            outer: Some(outer),
        }
    }

    pub fn new_builtins(builtin_functions: &[BuiltinFunction]) -> Self {
        let mut map = HashMap::new();

        BuiltinFunction::register(builtin_functions, &mut map);

        Environment { map, outer: None }
    }

    pub fn set(&mut self, identifier: String, value: Object) {
        self.map.insert(identifier, value);
    }

    pub fn get(&self, identifier: &String) -> Option<&Object> {
        let mut obj = self.map.get(identifier);
        if obj.is_none()
            && let Some(outer) = &self.outer
        {
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

    Pow,
    Len,
}

impl BuiltinFunction {
    const EXPORTS: [BuiltinFunction; 3] = [
        BuiltinFunction::ToString,
        BuiltinFunction::Print,
        BuiltinFunction::Println,
    ];

    pub fn name(&self) -> String {
        match self {
            BuiltinFunction::ToString => String::from("👁️‍🗨️"),
            BuiltinFunction::Print => String::from("🖨️"),
            BuiltinFunction::Println => String::from("🖨️↩️"),

            BuiltinFunction::Pow => String::from("💕"),
            BuiltinFunction::Len => String::from("📏"),
        }
    }

    pub fn function(&self) -> FunctionWrapper {
        match self {
            BuiltinFunction::ToString => Arc::new(BuiltinFunction::to_string) as FunctionWrapper,
            BuiltinFunction::Print => Arc::new(BuiltinFunction::print) as FunctionWrapper,
            BuiltinFunction::Println => Arc::new(BuiltinFunction::println) as FunctionWrapper,

            BuiltinFunction::Pow => Arc::new(BuiltinFunction::pow) as FunctionWrapper,
            BuiltinFunction::Len => Arc::new(BuiltinFunction::len) as FunctionWrapper,
        }
    }

    // api

    pub fn register_exports(map: &mut HashMap<String, Object>) {
        BuiltinFunction::register(&BuiltinFunction::EXPORTS, map);
    }

    pub fn register(functions: &[BuiltinFunction], map: &mut HashMap<String, Object>) {
        for function in functions {
            map.insert(function.name(), Object::BuiltinFunction(function.clone()));
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

        object_to_emoji(args.first().unwrap()).map(Object::string)
    }

    fn print(args: &[Object]) -> Result<Object, String> {
        if let Object::String(string, _) = BuiltinFunction::to_string(args)? {
            print!("{string}");
        }
        Ok(Object::Null)
    }

    fn println(args: &[Object]) -> Result<Object, String> {
        if let Object::String(string, _) = BuiltinFunction::to_string(args)? {
            println!("{string}");
        }
        Ok(Object::Null)
    }

    // builtin method implementations

    fn pow(args: &[Object]) -> Result<Object, String> {
        if args.len() != 2 {
            return Err(format!("Expected 2 argument(s), but got {}", args.len()));
        }

        let mut iterator = args.iter();
        let base = iterator.next().unwrap();
        let exp = iterator.next().unwrap();

        match base {
            Object::Integer(base, _) => match exp {
                Object::Integer(exp, _) if u32::try_from(*exp).is_ok() => base
                    .checked_pow(*exp as u32)
                    .map(Object::integer)
                    .ok_or_else(|| format!("Calculation overflow: {}.pow({})", base, exp)),
                Object::Float(exp, _) => Ok(Object::float((*base as f64).powf(*exp))),
                _ => Err(format!(
                    "Pow exponent must be Integer in {}~{} or Float: {}",
                    u32::MIN,
                    u32::MAX,
                    exp.inspect()
                )),
            },
            Object::Float(base, _) => match exp {
                Object::Integer(exp, _) if i32::try_from(*exp).is_ok() => {
                    Ok(Object::float(base.powi(*exp as i32)))
                }
                Object::Float(exp, _) => Ok(Object::float(base.powf(*exp))),
                _ => Err(format!(
                    "Pow exponent must be Integer in {}~{} or Float: {}",
                    i32::MIN,
                    i32::MAX,
                    exp.inspect()
                )),
            },
            _ => Err(format!(
                "Pow base number must be Integer/Float: {}",
                base.inspect()
            )),
        }
    }

    fn len(args: &[Object]) -> Result<Object, String> {
        if args.len() != 1 {
            return Err(format!("Expected 1 argument(s), but got {}", args.len()));
        }

        let length = match args.first().unwrap() {
            Object::String(value, _) => value.graphemes(true).count(),
            Object::List(value, _) => value.len(),
            Object::Map(value, _) => value.len(),
            object => return Err(format!("Expected string/list/map as instance, but got {:?}", object))
        };

        i64::try_from(length)
            .map(Object::integer)
            .map_err(|_| String::from("Calculation overflow: len()"))
    }
}
