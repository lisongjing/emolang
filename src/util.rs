#[derive(Debug)]
pub struct StatefulVector<T: PartialEq> {
    vector: Vec<T>,
    position: usize,
}

impl<T: PartialEq> Default for StatefulVector<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: PartialEq> StatefulVector<T> {
    pub fn new() -> StatefulVector<T> {
        StatefulVector {
            vector: vec![],
            position: 0,
        }
    }

    pub fn from_vec(vector: Vec<T>) -> StatefulVector<T> {
        StatefulVector {
            vector,
            position: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vector.is_empty()
    }

    pub fn first(&self) -> Option<&T> {
        self.vector.first()
    }

    pub fn last(&self) -> Option<&T> {
        self.vector.last()
    }

    pub fn current(&self) -> Option<&T> {
        if self.is_pos_valid(self.position) {
            Some(&self.vector[self.position])
        } else {
            None
        }
    }

    pub fn to_next(&mut self) -> Option<&T> {
        if self.has_next() {
            self.position += 1;
            Some(&self.vector[self.position])
        } else {
            None
        }
    }

    pub fn to_previous(&mut self) -> Option<&T> {
        if self.has_previous() {
            self.position -= 1;
            Some(&self.vector[self.position])
        } else {
            None
        }
    }

    pub fn is_next_eq(&self, expected_element: &T) -> bool {
        self.is_next_match(|ele| *ele == *expected_element)
    }

    pub fn is_next_match(&self, predicate: impl FnOnce(&T) -> bool) -> bool {
        if self.has_next() {
            predicate(&self.vector[self.position + 1])
        } else {
            false
        }
    }

    pub fn push(&mut self, element: T) {
        self.vector.push(element)
    }

    pub fn pop(&mut self) -> Option<T> {
        let ele = self.vector.pop();
        if ele.is_some() && self.position >= self.vector.len() {
            self.position = self.vector.len() - 1
        };
        ele
    }

    pub fn insert(&mut self, index: usize, element: T) {
        self.vector.insert(index, element)
    }

    pub fn remove(&mut self, index: usize) -> T {
        self.vector.remove(index)
    }

    pub fn has_next(&self) -> bool {
        self.is_pos_valid(self.position + 1)
    }

    pub fn has_previous(&self) -> bool {
        if self.position == 0 {
            false
        } else {
            self.is_pos_valid(self.position - 1)
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.vector.iter()
    }

    pub fn to_vec(self) -> Vec<T> {
        self.vector
    }

    fn is_pos_valid(&self, pos: usize) -> bool {
        if self.is_empty() {
            false
        } else {
            pos < self.vector.len()
        }
    }
}

pub mod emoji_convert {
    use crate::types::Object;

    const DOT: char = '.';
    const NULL_EMOJI: &str = "🈳";
    const DOT_EMOJI: &str = "\u{26aa}";
    const DIGITAL_EMOJI_SUFFIX: &str = "\u{fe0f}\u{20e3}";

    pub fn digital_emoji_to_char(emoji_char: &str) -> char {
        emoji_char.chars().next().unwrap()
    }

    pub fn dot_char() -> char {
        DOT
    }

    pub fn integer_to_emoji(integer: &i64) -> String {
        integer
            .to_string()
            .chars()
            .map(|char| format!("{char}{DIGITAL_EMOJI_SUFFIX}"))
            .collect()
    }

    pub fn float_to_emoji(float: &f64) -> String {
        float
            .to_string()
            .chars()
            .map(|char| {
                if char == '.' {
                    DOT_EMOJI.to_string()
                } else {
                    format!("{char}{DIGITAL_EMOJI_SUFFIX}")
                }
            })
            .collect()
    }

    pub fn boolean_to_emoji(boolean: &bool) -> String {
        String::from(if *boolean { "✔️" } else { "❌" })
    }

    pub fn object_to_emoji(object: &Object) -> Result<String, String> {
        let string = match object {
            Object::Integer(value) => integer_to_emoji(value),
            Object::Float(value) => float_to_emoji(value),
            Object::Boolean(value) => boolean_to_emoji(value),
            Object::String(value) => value.clone(),
            Object::Null => String::from(NULL_EMOJI),
            Object::List(value) => {
                let mut elements = vec![];
                for element in value {
                    elements.push(object_to_emoji(element)?);
                }
                format!("👉{}👈", elements.join("🦶 "))
            },
            Object::Map(value) => {
                let mut entries = vec![];
                for (key, val) in value {
                    entries.push(format!("{} ➡️ {}", object_to_emoji(key)?, object_to_emoji(val)?));
                }
                format!("🫸{}🫷", entries.join("🦶 "))
            },
            Object::ReturnValue(value) => object_to_emoji(value)?,
            _ => return Err(format!("Incompatible argument type with string: {:?}", object)),
        };
        Ok(string)
    }
}
