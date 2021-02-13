use std::collections::HashMap;
pub struct KvStore {
    cache: HashMap<String, String>,
}

impl KvStore {
    pub fn new() -> KvStore {
        KvStore {
            cache: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.cache.insert(key, value);
    }
    pub fn get(&mut self, key: String) -> Option<String> {
        match self.cache.get(&key) {
            Some(value) => Some(value.to_string()),
            None => None,
        }
    }
    pub fn remove(&mut self, key: String) {
        self.cache.remove(&key);
    }
}
