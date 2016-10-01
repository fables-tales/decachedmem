use memcached::types::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Store {
    kv: HashMap<MemcachedKey, MemcachedValue>,
}

impl Store {
    pub fn new() -> Store {
        Store { kv: HashMap::new() }
    }

    pub fn set(&mut self, key: &MemcachedKey, value: MemcachedValue) {
        self.kv.insert(key.clone(), value);
    }

    pub fn get(&mut self, key: &MemcachedKey) -> Option<&MemcachedValue> {
        self.kv.get(key)
    }
}
