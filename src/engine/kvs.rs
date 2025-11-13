use crate::error::{KvsError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const COMPACTION_THRESHOLD: u64 = 1024 * 1024; // 1MB

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are persisted to a log file on disk.
/// The log file is named `wal.log`.
/// An in-memory `HashMap` is used to index the log file.
///
/// Example:
///
/// ```rust
/// use std::env::current_dir;
/// use kvs::{KvStore, Result};
///
/// fn main() -> Result<()> {
///     let mut store = KvStore::open(current_dir()?)?;
///     store.set("key".to_owned(), "value".to_owned())?;
///     let val = store.get("key".to_owned())?;
///     assert_eq!(val, Some("value".to_owned()));
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct KvStore(Arc<Mutex<KvStoreInner>>);

pub struct KvStoreInner {
    path: PathBuf,
    writer: BufWriter<File>,
    reader: BufReader<File>,
    index: HashMap<String, CommandPos>,
    stale_bytes: u64,
}

#[derive(Debug, Clone, Copy)]
struct CommandPos {
    pos: u64,
    len: u64,
}

impl KvStoreInner {
    fn build_index(reader_file: &File) -> Result<(HashMap<String, CommandPos>, u64)> {
        let mut index = HashMap::new();
        let mut stale_bytes = 0;
        let mut reader = BufReader::new(reader_file);
        let mut pos = reader.seek(SeekFrom::Start(0))?;
        let mut stream = serde_json::Deserializer::from_reader(&mut reader).into_iter::<Command>();

        while let Some(cmd) = stream.next() {
            let new_pos = stream.byte_offset() as u64;
            let len = new_pos - pos;
            match cmd? {
                Command::Set { key, .. } => {
                    if let Some(old_cmd) = index.insert(key, CommandPos { pos, len }) {
                        stale_bytes += old_cmd.len;
                    }
                }
                Command::Remove { key } => {
                    if let Some(old_cmd) = index.remove(&key) {
                        stale_bytes += old_cmd.len;
                    }
                    stale_bytes += len;
                }
            }
            pos = new_pos;
        }
        Ok((index, stale_bytes))
    }

    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    /// The command is written to the log file and the index is updated.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::Set {
            key: key.clone(),
            value,
        };

        let pos = self.writer.stream_position()?;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;
        let new_pos = self.writer.stream_position()?;
        let len = new_pos - pos;

        if let Some(old_cmd) = self.index.insert(key, CommandPos { pos, len }) {
            self.stale_bytes += old_cmd.len;
        }

        if self.stale_bytes > COMPACTION_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    /// The value is read from the log file.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(cmd_pos) = self.index.get(&key) {
            self.reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            let cmd_reader = self.reader.get_mut().take(cmd_pos.len);
            let cmd = serde_json::from_reader(cmd_reader)?;

            if let Command::Set { value, .. } = cmd {
                Ok(Some(value))
            } else {
                Err(KvsError::UnexpectedCommandType)
            }
        } else {
            Ok(None)
        }
    }

    /// Remove a given key.
    ///
    /// A `Remove` command is written to the log file and the key is removed from the index.
    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let cmd = Command::Remove { key: key.clone() };
            let pos = self.writer.stream_position()?;
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;

            let new_pos = self.writer.stream_position()?;
            let len = new_pos - pos;

            if let Some(old_cmd) = self.index.remove(&key) {
                self.stale_bytes += old_cmd.len;
                self.stale_bytes += len;
            }

            if self.stale_bytes > COMPACTION_THRESHOLD {
                self.compact()?;
            }

            Ok(())
        } else {
            Err(KvsError::KeyNotFound)
        }
    }

    fn compact(&mut self) -> Result<()> {
        // 1. Create new log file and a new index
        let compaction_path = self.path.join("wal.log.compact");
        let mut compaction_writer = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .write(true)
                .open(&compaction_path)?,
        );
        let mut new_index = HashMap::new();

        // 2. Write current values to new log and build new index
        for key in self.index.keys() {
            let cmd_pos = self.index.get(key).unwrap();
            self.reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            let mut cmd_reader = self.reader.get_mut().take(cmd_pos.len);

            let pos = compaction_writer.stream_position()?;
            std::io::copy(&mut cmd_reader, &mut compaction_writer)?;
            let new_pos = compaction_writer.stream_position()?;
            new_index.insert(key.clone(), CommandPos { pos, len: new_pos - pos });
        }
        compaction_writer.flush()?;

        // 3. Atomically replace old log with new
        std::fs::rename(&compaction_path, self.path.join("wal.log"))?;

        // 4. Re-open writer and reader, update index and stale_bytes
        self.writer = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .open(self.path.join("wal.log"))?,
        );
        self.writer.seek(SeekFrom::End(0))?;
        self.reader = BufReader::new(File::open(self.path.join("wal.log"))?);
        self.index = new_index;
        self.stale_bytes = 0;

        Ok(())
    }
}

impl KvStore {
    /// Opens a `KvStore` with the given path.
    ///
    /// This will create a new directory if the given one does not exist.
    /// It will also create a `wal.log` file if it does not exist.
    /// The index will be built from the log file.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        std::fs::create_dir_all(&path)?;
        let log_path = path.join("wal.log");

        let writer_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&log_path)?;
        let reader_file = File::open(&log_path)?;

        let (index, stale_bytes) = KvStoreInner::build_index(&reader_file)?;

        let mut writer = BufWriter::new(writer_file);
        writer.seek(SeekFrom::End(0))?;

        let inner = KvStoreInner {
            path,
            writer,
            reader: BufReader::new(reader_file),
            index,
            stale_bytes,
        };

        Ok(KvStore(Arc::new(Mutex::new(inner))))
    }

    /// Sets the value of a string key to a string.
    pub fn set(&self, key: String, value: String) -> Result<()> {
        let mut inner = self.0.lock().unwrap();
        inner.set(key, value)
    }

    /// Gets the string value of a given string key.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        let mut inner = self.0.lock().unwrap();
        inner.get(key)
    }

    /// Remove a given key.
    pub fn remove(&self, key: String) -> Result<()> {
        let mut inner = self.0.lock().unwrap();
        inner.remove(key)
    }
}

impl super::KvsEngine for KvStore {
    fn set(&self, key: String, value: String) -> Result<()> {
        KvStore::set(self, key, value)
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        KvStore::get(self, key)
    }

    fn remove(&self, key: String) -> Result<()> {
        KvStore::remove(self, key)
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}
