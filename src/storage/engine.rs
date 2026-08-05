use std::{fs::{self, create_dir_all}, io, path::{Path, PathBuf}};
use crate::{storage::{index::Index, segment::{Segment}}, store::log::Command};

pub struct StorageEngine {
    dir : PathBuf,
    seg : Vec<Segment>,
    active : Segment,
    index : Index,
    max_segment_size : u64
}

impl StorageEngine {
     
     pub fn open(dir : impl AsRef<Path>, max_segment_size : u64) -> io::Result<Self> {
        
        let dir_path = dir.as_ref().to_path_buf();
        create_dir_all(&dir_path)?;

        let mut segment_ids = Vec::new();
        
        for entry in fs::read_dir(&dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(id) = parse_segment_id(&path) {
                segment_ids.push(id);
            }
        }
        
        segment_ids.sort_unstable();
        
        let (sealed_segments, active_segment) = if segment_ids.is_empty() {
            (Vec::new(), Segment::new(0, &dir_path, true)?)
        } else {
            let active_id =  segment_ids.pop().unwrap();

            let mut sealed = Vec::new();
            for id in segment_ids {
                sealed.push(Segment::read_only(id, &dir_path)?);
            }
            let acitve = Segment::new(active_id, &dir_path, true)?;
            (sealed,acitve)
        };

        
        let mut index = Index::new();
        for segment in &sealed_segments {
            let entries = segment.read_all()?;
            for (offset, cmd) in entries {
                match cmd {
                    Command::Put { key, .. } => index.set(&key, segment.id, offset),
                    Command::Delete { key } => index.remove(&key),
                    _=> {}
                }
            }

        }
        let active_entries = active_segment.read_all()?;
        for (offset, cmd) in active_entries {
        
        match cmd {
                 Command::Put { key, .. } => index.set(&key, active_segment.id, offset),
                 Command::Delete { key } => index.remove(&key),
                 _ => {}
                  }
                }

        Ok(Self { dir: dir_path, seg: sealed_segments, active: active_segment, index, max_segment_size})
     }

     pub fn put (&mut self, key: String , value : String) -> io::Result<()> {
        
        let cmd = Command::Put { key: key.clone(), value };
        
        let target_offset = self.active.append_segment(&cmd)?;
        self.index.set(&key, self.active.id, target_offset);

        if self.active.current_offset > self.max_segment_size {
            self.rotate_segments()?;
        }

        Ok(())
     }     

     pub fn get (&self, key: &str) -> io::Result<Option<String>> {
         
         let location = match self.index.get(key) {
            Some(loc) => loc,
            None => return Ok(None),

         };
         let cmd = if location.segment_id == self.active.id {
            self.active.read_at(location.offset)?
         } else {
             let segment = self.seg.iter().find(|s| s.id == location.segment_id).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Segment File Not Found")
             })?;

             segment.read_at(location.offset)?
         
         };
       
            match cmd {
                  Command::Put { value, .. } => Ok(Some(value)),
                         _ => Ok(None)
}
          }
     
     pub fn rotate_segments (&mut self) -> io::Result<()> {
        let next_segment_id = self.active.id + 1;

        let new_active_segment =  Segment::new(next_segment_id, &self.dir, true)?;
        let mut old_active_segment = std::mem::replace(&mut self.active, new_active_segment);

        old_active_segment.is_active = false;
        self.seg.push(old_active_segment);

        Ok(())
     }
            
     pub fn delete (&mut self, key: String) -> io::Result<()> {
        let cmd = Command::Delete { key: key.clone() };
        self.active.append_segment(&cmd)?;
        self.index.remove(&key);

        if self.active.current_offset >= self.max_segment_size {
            self.rotate_segments()?;
        }
      Ok(())
     }

    }
     

fn parse_segment_id(path: &Path) -> Option<u64> {
    let file_name = path.file_name()?.to_str()?;
    if file_name.starts_with("segment_") && file_name.ends_with(".log") {
        let id_str = file_name.strip_prefix("segment_")?.strip_suffix(".log")?;
        id_str.parse::<u64>().ok()
    }  else {
        None
    }
}