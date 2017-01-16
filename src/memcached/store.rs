use memcached;
use std::collections::HashMap;

pub struct Store {
    values: HashMap<memcached::Key, Vec<u8>>,
}

impl Store {
    pub fn new() -> Self {
        Store { values: HashMap::new() }
    }
    pub fn get(&self, key: &memcached::Key) -> Option<Vec<u8>> {
        self.values.get(key).map(|vec| vec.clone())
    }

    pub fn set(&mut self, key: memcached::Key, value: Vec<u8>) {
        self.values.insert(key, value);
    }
}
