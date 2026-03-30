use std::{
    collections::HashMap,
    hash::Hash,
    sync::Arc,
};

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
            Object::List(value) => format!(
                "[{}]",
                value
                    .iter()
                    .map(|obj| obj.inspect())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Object::Map(value) => format!(
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
        match self {
            Object::Integer(_) => Environment::new_builtins(&[BuiltinFunction::Pow]),
            Object::Float(_) => Environment::new_builtins(&[BuiltinFunction::Pow]),
            Object::Boolean(_) => todo!(),
            Object::String(_) => todo!(),
            Object::List(_) => todo!(),
            Object::Map(_) => todo!(),
            _ => Environment::new_empty(),
        }
    }
}

impl Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Object::Integer(value) => {
                0u32.hash(state);
                value.hash(state);
            }
            Object::Float(value) => {
                1u32.hash(state);
                OrderedFloat(*value).hash(state);
            }
            Object::Boolean(value) => {
                2u32.hash(state);
                value.hash(state);
            }
            Object::String(value) => {
                3u32.hash(state);
                value.hash(state);
            }
            Object::Null => {
                4u32.hash(state);
            }
            Object::List(elements) => {
                5u32.hash(state);
                for element in elements {
                    element.hash(state);
                }
            }
            Object::Map(entries) => {
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
    pub fn new_empty() -> Self {
        Environment {
            map: HashMap::new(),
            outer: None,
        }
    }

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
        }
    }

    pub fn function(&self) -> FunctionWrapper {
        match self {
            BuiltinFunction::ToString => Arc::new(BuiltinFunction::to_string) as FunctionWrapper,
            BuiltinFunction::Print => Arc::new(BuiltinFunction::print) as FunctionWrapper,
            BuiltinFunction::Println => Arc::new(BuiltinFunction::println) as FunctionWrapper,

            BuiltinFunction::Pow => Arc::new(BuiltinFunction::pow) as FunctionWrapper,
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

        object_to_emoji(args.first().unwrap()).map(Object::String)
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

    // builtin method implementations

    fn pow(args: &[Object]) -> Result<Object, String> {
        if args.len() != 2 {
            return Err(format!("Expected 2 argument(s), but got {}", args.len()));
        }

        let base = args.iter().next().unwrap();
        let exp = args.iter().next().unwrap();

        match base {
            Object::Integer(base) => match exp {
                Object::Integer(exp) if u32::try_from(*exp).is_ok() => base
                    .checked_pow(*exp as u32)
                    .map(Object::Integer)
                    .ok_or_else(|| format!("Calculation overflow: {}.pow({})", base, exp)),
                Object::Float(exp) => Ok(Object::Float((*base as f64).powf(*exp))),
                _ => Err(format!(
                    "Pow exponent must be Integer in {}~{} or Float: {}",
                    u32::MIN,
                    u32::MAX,
                    exp.inspect()
                )),
            },
            Object::Float(base) => match exp {
                Object::Integer(exp) if i32::try_from(*exp).is_ok() => {
                    Ok(Object::Float(base.powi(*exp as i32)))
                }
                Object::Float(exp) => Ok(Object::Float(base.powf(*exp))),
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
}
