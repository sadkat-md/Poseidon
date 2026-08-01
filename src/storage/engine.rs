use std::{io, path::PathBuf};

use crate::storage::{index::Index, segment::Segment};



pub struct StorageEngine {
    dir : PathBuf,
    seg : Vec<Segment>,
    active : Segment,
    index : Index,
    max_segment_size : u64
}

impl StorageEngine {
     pub fn open(dir : PathBuf, segment_size : u64) -> io::Result<Self> {
        
        
        Ok()
     }
}