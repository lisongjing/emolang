use std::{
    collections::HashMap, hash::Hash, sync::{Arc, LazyLock, Mutex}
};

use ordered_float::OrderedFloat;
use unicode_segmentation::UnicodeSegmentation;

use crate::{types::Node, util::emoji_convert::object_to_emoji};

#[derive(Debug, Clone)]
pub struct Object {
    value: ObjectValue,
    associated_env: Environment,
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Object {}

impl Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match &self.value {
            ObjectValue::Integer(value) => {
                0u32.hash(state);
                value.hash(state);
            }
            ObjectValue::Float(value) => {
                1u32.hash(state);
                OrderedFloat(*value).hash(state);
            }
            ObjectValue::Boolean(value) => {
                2u32.hash(state);
                value.hash(state);
            }
            ObjectValue::String(value) => {
                3u32.hash(state);
                value.hash(state);
            }
            ObjectValue::Null => {
                4u32.hash(state);
            }
            ObjectValue::List(elements) => {
                5u32.hash(state);
                for element in elements {
                    element.hash(state);
                }
            }
            ObjectValue::Map(entries) => {
                6u32.hash(state);
                for (key, value) in entries {
                    key.hash(state);
                    value.hash(state);
                }
            }
            ObjectValue::Function {
                parameters: _,
                body,
                env: _,
            } => {
                7u32.hash(state);
                body.string().hash(state);
            }
            ObjectValue::BuiltinFunction(value) => {
                8u32.hash(state);
                value.name().hash(state);
            }
            ObjectValue::Reference(value) => {
                9u32.hash(state);
                (*value.lock().unwrap()).hash(state);
            }
            ObjectValue::ReturnValue(value) => {
                10u32.hash(state);
                value.hash(state);
            }
            ObjectValue::Break(value) => {
                11u32.hash(state);
                if let Some(val) = value {
                    val.hash(state);
                }
            }
            ObjectValue::Continue => {
                12u32.hash(state);
            }
        }
    }
}

impl Object {
    pub fn inspect(&self) -> String {
        match &self.value {
            ObjectValue::Integer(val) => val.to_string(),
            ObjectValue::Float(val) => val.to_string(),
            ObjectValue::Boolean(val) => val.to_string(),
            ObjectValue::String(val) => format!("\"{}\"", val),
            ObjectValue::Null => "null".to_string(),
            ObjectValue::List(val) => format!(
                "[{}]",
                val
                    .iter()
                    .map(|obj| obj.inspect())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ObjectValue::Map(val) => format!(
                "{{{}}}",
                val
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.inspect(), v.inspect()))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ObjectValue::Function {
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
            ObjectValue::BuiltinFunction(val) => format!(
                "{}(args...){{ //builtin implementation }}",
                val.name()
            ),
            ObjectValue::Reference(val) => (*val.lock().unwrap()).inspect(),
            ObjectValue::ReturnValue(val) => val.inspect(),
            ObjectValue::Break(val) => val.clone().map_or("!".to_string(), |v| v.inspect()),
            ObjectValue::Continue => "!".to_string(),
        }
    }

    pub fn value(&self) -> &ObjectValue {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut ObjectValue {
        &mut self.value
    }

    pub fn associated_env(&self) -> &Environment {
        &self.associated_env
    }

    pub fn associated_env_mut(&mut self) -> &mut Environment {
        &mut self.associated_env
    }

    fn set_self_in_assoc_env(mut obj: Object) -> Object {
        obj.associated_env.set("🈯".to_string(), obj.clone());
        obj
    }

    pub fn new_integer(value: i64) -> Object {
        Self::set_self_in_assoc_env(
            Object {
                value: ObjectValue::Integer(value),
                associated_env: Environment::new_builtins(&[BuiltinFunction::Pow]),
            }
        )
    }

    pub fn new_float(value: f64) -> Object {
        Self::set_self_in_assoc_env(
            Object {
                value: ObjectValue::Float(value),
                associated_env: Environment::new_builtins(&[BuiltinFunction::Pow]),
            }
        )
    }

    pub fn new_boolean(value: bool) -> Object {
        if value {TRUE.clone()} else {FALSE.clone()}
    }

    pub fn new_string(value: String) -> Object {
        Self::set_self_in_assoc_env(
            Object {
                value: ObjectValue::String(value),
                associated_env: Environment::new_builtins(&[BuiltinFunction::Len]),
            }
        )
    }

    pub fn new_null() -> Object {
        NULL.clone()
    }

    pub fn new_list(value: Vec<Object>) -> Object {
        Self::set_self_in_assoc_env(
            Object {
                value: ObjectValue::List(value),
                associated_env: Environment::new_builtins(&[BuiltinFunction::Len]),
            }
        )
    }

    pub fn new_map(value: HashMap<Object, Object>) -> Object {
        Self::set_self_in_assoc_env(
            Object {
                value: ObjectValue::Map(value),
                associated_env: Environment::new_builtins(&[BuiltinFunction::Len]),
            }
        )
    }

    pub fn new_butlin_function(value: BuiltinFunction) -> Object {
        Self::set_self_in_assoc_env(
            Object {
                value: ObjectValue::BuiltinFunction(value),
                associated_env: Environment::new_builtins(&[]),
            }
        )
    }

    pub fn new_function(parameters: Vec<Node>, body: Box<Node>, env: Environment) -> Object {
        Self::set_self_in_assoc_env(
            Object {
                value: ObjectValue::Function {
                    parameters,
                    body,
                    env: Box::new(env),
                },
                associated_env: Environment::new_builtins(&[]),
            }
        )
    }

    pub fn new_return_value(value: Object) -> Object {
        Self::set_self_in_assoc_env(
            Object {
                value: ObjectValue::ReturnValue(Box::new(value)),
                associated_env: Environment::new_builtins(&[]),
            }
        )
    }
}


#[derive(Debug, Clone)]
pub enum ObjectValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Null,
    List(Vec<Object>),
    Map(HashMap<Object, Object>),
    Function {
        parameters: Vec<Node>,
        body: Box<Node>,
        env: Box<Environment>,
    },
    BuiltinFunction(BuiltinFunction),
    Reference(Arc<Mutex<Object>>),
    ReturnValue(Box<Object>),
    Break(Option<Box<Object>>),
    Continue,
}

impl PartialEq for ObjectValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ObjectValue::Integer(a), ObjectValue::Integer(b)) => a == b,
            (ObjectValue::Float(a), ObjectValue::Float(b)) => a == b,
            (ObjectValue::Boolean(a), ObjectValue::Boolean(b)) => a == b,
            (ObjectValue::String(a), ObjectValue::String(b)) => a == b,
            (ObjectValue::Null, ObjectValue::Null) => true,
            (ObjectValue::List(a), ObjectValue::List(b)) => a == b,
            (ObjectValue::Map(a), ObjectValue::Map(b)) => a == b,
            (ObjectValue::Function { parameters: ap, body: ab, env: _ }, ObjectValue::Function { parameters: bp, body: bb, env: _ }) => ap == bp && ab == bb,
            (ObjectValue::BuiltinFunction(a), ObjectValue::BuiltinFunction(b)) => a == b,
            (ObjectValue::Reference(a), ObjectValue::Reference(b)) => *a.lock().unwrap() == *b.lock().unwrap(),
            (ObjectValue::ReturnValue(a), ObjectValue::ReturnValue(b)) => a == b,
            (ObjectValue::Break(a), ObjectValue::Break(b)) => a == b,
            (ObjectValue::Continue, ObjectValue::Continue) => true,
            _ => false,
        }
    }
}


pub static TRUE: LazyLock<Object> = LazyLock::new(|| Object::set_self_in_assoc_env(
    Object {
        value: ObjectValue::Boolean(true),
        associated_env: Environment::new_builtins(&[]),
    }
));

pub static FALSE: LazyLock<Object> = LazyLock::new(|| Object::set_self_in_assoc_env(
    Object {
        value: ObjectValue::Boolean(false),
        associated_env: Environment::new_builtins(&[]),
    }
));

pub static NULL: LazyLock<Object> = LazyLock::new(|| Object::set_self_in_assoc_env(
    Object {
        value: ObjectValue::Null,
        associated_env: Environment::new_builtins(&[]),
    }
));



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

    pub fn get_mut(&mut self, identifier: &String) -> Option<&mut Object> {
        let mut obj = self.map.get_mut(identifier);
        if obj.is_none()
            && let Some(outer) = &mut self.outer
        {
            obj = outer.get_mut(identifier);
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
            map.insert(function.name(), Object::new_butlin_function(function.clone()));
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

        object_to_emoji(args.first().unwrap()).map(Object::new_string)
    }

    fn print(args: &[Object]) -> Result<Object, String> {
        if let ObjectValue::String(string) = BuiltinFunction::to_string(args)?.value() {
            print!("{string}");
        }
        Ok(Object::new_null())
    }

    fn println(args: &[Object]) -> Result<Object, String> {
        if let ObjectValue::String(string) = BuiltinFunction::to_string(args)?.value() {
            println!("{string}");
        }
        Ok(Object::new_null())
    }

    // builtin method implementations

    fn pow(args: &[Object]) -> Result<Object, String> {
        if args.len() != 2 {
            return Err(format!("Expected 2 argument(s), but got {}", args.len()));
        }

        let mut iterator = args.iter();
        let base = iterator.next().unwrap();
        let exp = iterator.next().unwrap();

        match base.value() {
            ObjectValue::Integer(base) => match exp.value() {
                ObjectValue::Integer(exp) if u32::try_from(*exp).is_ok() => base
                    .checked_pow(*exp as u32)
                    .map(Object::new_integer)
                    .ok_or_else(|| format!("Calculation overflow: {}.pow({})", base, exp)),
                ObjectValue::Float(exp) => Ok(Object::new_float((*base as f64).powf(*exp))),
                _ => Err(format!(
                    "Pow exponent must be Integer in {}~{} or Float: {}",
                    u32::MIN,
                    u32::MAX,
                    exp.inspect()
                )),
            },
            ObjectValue::Float(base) => match exp.value() {
                ObjectValue::Integer(exp) if i32::try_from(*exp).is_ok() => {
                    Ok(Object::new_float(base.powi(*exp as i32)))
                }
                ObjectValue::Float(exp) => Ok(Object::new_float(base.powf(*exp))),
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

        let length = match args.first().unwrap().value() {
            ObjectValue::String(value) => value.graphemes(true).count(),
            ObjectValue::List(value) => value.len(),
            ObjectValue::Map(value) => value.len(),
            object => return Err(format!("Expected string/list/map as instance, but got {:?}", object))
        };

        i64::try_from(length)
            .map(Object::new_integer)
            .map_err(|_| String::from("Calculation overflow: len()"))
    }
}
