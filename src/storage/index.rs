use::std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Index {
    map : HashMap<String, EntryLocation>,
}

#[derive(Debug,Clone,Copy)]
pub struct EntryLocation {
    pub segment_id : u64,
    pub offset : u64
}

impl Index {
    pub fn new() -> Self {
        Self {map : HashMap::new()}
    }
    pub fn get(&self, key: &str) -> Option<EntryLocation> {
         self.map.get(key).copied()
    }
    pub fn set(&mut self, key : &str, segment_id : u64, offset: u64) {
        self.map.insert(key.to_string(), EntryLocation { segment_id, offset });
    }
    pub fn remove (&mut self, key: &str)  {
         self.map.remove(key);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &EntryLocation)> {
        self.map.iter()
    }
}