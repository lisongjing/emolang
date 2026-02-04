use std::any::{Any, TypeId};

use crate::types::{object::*, Node};

pub fn eval(node: Box<dyn Node>) -> Box<dyn Object> {
    Box::new(EmoNull {})
}