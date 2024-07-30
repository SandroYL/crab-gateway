use std::collections::{
    hash_map::{Entry, Iter, IterMut},
    HashMap,
};

pub struct CaseSenseMap {
    inner: HashMap<String, Vec<String>>,
}

impl CaseSenseMap {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn get(&self, input: String) -> Option<String> {
        Self::format_value(self.inner.get(&input.to_lowercase()))
    }

    pub fn insert(&mut self, key: String, value: String) -> Option<String> {
        let origin_value = self.inner.remove(&key);
        self.inner.insert(key, vec![value]);
        Self::format_value(origin_value.as_ref())
    }

    pub fn append(&mut self, key: String, value: String) {
        let origin_values = self.entry(key).or_insert(Vec::new());
        (*origin_values).push(value);
    }

    pub fn entry(&mut self, key: String) -> Entry<String, Vec<String>> {
        self.inner.entry(key.to_lowercase())
    }

    pub fn remove(&mut self, key: String) -> Option<Vec<String>> {
        self.inner.remove(&key.to_lowercase())
    }

    pub fn iter(&self) -> Iter<String, Vec<String>> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<String, Vec<String>> {
        self.inner.iter_mut()
    }

    #[inline]
    fn format_value(value: Option<&Vec<String>>) -> Option<String> {
        return if value.is_none() {
            None
        } else {
            Some(format!("[{}]", value.unwrap().join(",")))
        };
    }
}

