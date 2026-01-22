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
        self.is_pos_valid(self.position - 1)
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
