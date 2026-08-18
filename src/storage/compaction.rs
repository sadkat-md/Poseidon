use::std::{collections::HashMap, fs, io, path::Path};

use crate::{
    storage::{index::{Index}, segment::{Segment}}, store::log::Command,
};

pub fn compaction(sealed: &[Segment], dir : &Path, next_id : u64) -> io::Result<(Segment, Index)> {
       
       let mut latest: HashMap<String, Command> = HashMap::new();

       for segment in sealed {
           for (_, cmd) in segment.read_all()?{
              match cmd {
                Command::Put { ref key, .. } => {
                    latest.insert(key.clone(),cmd);
                }
                Command::Delete { ref key } => {
                    latest.insert(key.clone(), cmd);
                }
              _=> {}
              }
           }
       }
       let mut compacted_segment = Segment::new(next_id, dir, true)?;
       let mut new_index  = Index::new();

       for(_,cmd) in &latest {
           if let Command::Put { key, .. } = cmd {
            let offset = compacted_segment.append_segment(cmd)?;
            new_index.set(key, next_id, offset);
           }
       }

       Ok((compacted_segment, new_index))
}

pub fn delete_old_segments (sealed: &[Segment]) -> io::Result<()> {
       for segment in sealed {
           fs::remove_file(&segment.path)?;
       }
       Ok(())
}