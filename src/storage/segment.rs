use crate::store::log::Command;
use crc32fast::Hasher;
use std::{
    fs::{File, OpenOptions, create_dir_all}, io::{self, BufWriter, Read, Seek, SeekFrom, Write}, path::{Path, PathBuf},
};

pub struct Segment {
    pub id: u64,             // segment_0, segment_1, etc.
    pub path: PathBuf,       // "storage/segment_0.log"
    writer: BufWriter<File>, // for appending new entries
    pub current_offset: u64, // byte position of the next write
    pub is_active: bool,     // only the active segment accepts writes
}

impl Segment {
    pub fn new(id: u64, dir_path: impl AsRef<Path>, is_active: bool) -> io::Result<Self> {
        let dir_path = dir_path.as_ref();
        create_dir_all(dir_path)?;
        let path = dir_path.join(format!("segment_{}.log", id));

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&path)?;
        let current_offset = file.metadata()?.len();

        let writer = BufWriter::new(file);

        Ok(Self {
            id,
            path,
            writer,
            current_offset,
            is_active,
        })
    }

    pub fn read_only(id: u64, dir_path: impl AsRef<Path>) -> io::Result<Self> {
        let dir_path = dir_path.as_ref();
        let path = dir_path.join(format!("segment_{}.log", id));

        let file = OpenOptions::new().read(true).open(&path)?;
        let current_offset = file.metadata()?.len();
        let writer = BufWriter::new(file);

        Ok(Self {
            id,
            path,
            writer,
            current_offset,
            is_active: false,
        })
    }

    pub fn append_segment(&mut self, cmd: &Command) -> io::Result<u64> {
        if !self.is_active {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "cannot write to a read only segment",
            ));
        }

        let start_offset = self.current_offset;
        let clean_cmd = cmd.serialize();
        let payload_len = clean_cmd.len() as u32;

        let mut hasher = Hasher::new();
        hasher.update(&clean_cmd);

        let checksum = hasher.finalize();

        self.writer.write_all(&checksum.to_le_bytes())?;
        self.writer.write_all(&payload_len.to_le_bytes())?;
        self.writer.write_all(&clean_cmd)?;

        self.writer.flush()?;

        let bytes_written = 8 + clean_cmd.len() as u64;
        self.current_offset += bytes_written;

        Ok(start_offset)
    }

    pub fn read_at(&self, offset: u64) -> io::Result<Command> {
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(offset))?;

        let mut checksum_bytes = [0u8; 4];
        file.read_exact(&mut checksum_bytes)?;
        let stored_checksum = u32::from_le_bytes(checksum_bytes);

        let mut len_bytes = [0u8; 4];
        file.read_exact(&mut len_bytes)?;
        let payload_len = u32::from_le_bytes(len_bytes) as usize;

        let mut payload = vec![0u8; payload_len];
        file.read_exact(&mut payload)?;

        let mut hasher = Hasher::new();
        hasher.update(&payload);
        let computed = hasher.finalize();

        if stored_checksum != computed {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Mismatched Checksum!",
            ));
        }

        Command::deserialized(&payload).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Unable to deserialize the data!",
            )
        })
    }

    pub fn read_all(&self) -> io::Result<Vec<(u64, Command)>> {
         let mut entries : Vec<(u64, Command)> = Vec::new();
         let mut file =  File::open(&self.path)?;
         let mut offset: u64 = 0;

         loop {
                
                let mut checksum_bytes = [0u8; 4];
                
                match file.read_exact(&mut checksum_bytes) {
                    Ok(_) => {}
                    Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                    Err(e) => return Err(e),
                }
                
                let stored_checksum = u32::from_le_bytes(checksum_bytes);

                let mut len_bytes = [0u8; 4];
                file.read_exact(&mut len_bytes)?;
                
                let payload_len =  u32::from_le_bytes(len_bytes) as usize;

                let mut payload = vec![0u8; payload_len];
                file.read_exact(&mut payload)?;

                let mut hasher = Hasher::new();
                hasher.update(&payload);
                let computed = hasher.finalize();

                if stored_checksum != computed{
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "checksum Mismatched!"))
                }

                let cmd = Command::deserialized(&payload).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "unable to deserialize data!")})?;

                entries.push((offset,cmd));
                offset += 8 + payload_len as u64;

         };
      Ok(entries)
    }
}
