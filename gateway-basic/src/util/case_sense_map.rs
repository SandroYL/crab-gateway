use std::collections::{
    hash_map::{Entry, Iter, IterMut},
    HashMap, HashSet,
};

pub struct CaseSenseMap {
    inner: HashMap<String, HashSet<String>>,
}

impl CaseSenseMap {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn get(&self, input: &str) -> Option<String> {
        Self::format_value(self.inner.get(&input.to_lowercase()))
    }

    pub fn contains(&self, input: &str) -> bool {
        self.inner.contains_key(&input.to_lowercase())
    }

    pub fn insert(&mut self, mut key: String, value: String) -> Option<String> {
        key = key.to_lowercase();
        let mut init_set = HashSet::new();
        let old_set = self.get(&key);
        init_set.insert(value);
        self.inner.insert(key, init_set.clone());
        old_set
    }

    pub fn append(&mut self, key: String, value: String) {
        let origin_values = self.entry(key).or_insert(HashSet::new());
        (*origin_values).insert(value);
    }

    pub fn entry(&mut self, key: String) -> Entry<String, HashSet<String>> {
        self.inner.entry(key.to_lowercase())
    }

    pub fn remove(&mut self, key: String) -> Option<HashSet<String>> {
        self.inner.remove(&key.to_lowercase())
    }


    pub fn remove_value(&mut self, key: String, value: String) {
        let v = self.inner.entry(key.to_lowercase()).or_default();
        v.remove(&value);
        if v.len() == 0 {
            self.inner.remove(&key);
        }
    }

    pub fn iter(&self) -> Iter<String, HashSet<String>> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<String, HashSet<String>> {
        self.inner.iter_mut()
    }

    #[inline]
    fn format_value(value: Option<&HashSet<String>>) -> Option<String> {
        if let Some(vset) = value {
            let capacity = vset.iter().map(|s| s.len())
                .sum::<usize>() + vset.len() + 1;
            let mut print_str = String::with_capacity(capacity);
            print_str.push_str("[");
            for (i, s) in vset.iter().enumerate() {
                if i > 0 {
                    print_str.push_str(",");
                }
                print_str.push_str(s);
            }
            print_str.push_str("]");
            Some(print_str)
        } else {
            None
        }
    }
}

