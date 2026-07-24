use::std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Index {
    map : HashMap<String, u64>,
}

impl Index {
    pub fn new() -> Self {
        Self {map : HashMap::new()}
    }
    pub fn get(&self, key: &str) -> Option<u64> {
         self.map.get(key).copied()
    }
    pub fn set(&mut self, key : &str, offset: u64) {
        self.map.insert(key.to_string(), offset);
    }
    pub fn remove (&mut self, key: &str)  {
         self.map.remove(key);
    }
}