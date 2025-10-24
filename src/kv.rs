use crate::error::{KvsError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

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
/// use kvs::KvStore;
///
/// let mut store = KvStore::open(current_dir()?)?;
/// store.set("key".to_owned(), "value".to_owned())?;
/// let val = store.get("key".to_owned())?;
/// assert_eq!(val, Some("value".to_owned()));
/// ```
pub struct KvStore {
    writer: BufWriter<File>,
    reader: BufReader<File>,
    index: HashMap<String, CommandPos>,
}

#[derive(Debug, Clone, Copy)]
struct CommandPos {
    pos: u64,
    len: u64,
}

impl KvStore {
    /// Opens a `KvStore` with the given path.
    ///
    /// This will create a new directory if the given one does not exist.
    /// It will also create a `wal.log` file if it does not exist.
    /// The index will be built from the log file.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let mut path = path.into();
        std::fs::create_dir_all(&path)?;
        path.push("wal.log");

        let writer_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)?;
        let reader_file = File::open(&path)?;

        let index = Self::build_index(&reader_file)?;

        let mut writer = BufWriter::new(writer_file);
        writer.seek(SeekFrom::End(0))?;

        Ok(KvStore {
            writer,
            reader: BufReader::new(reader_file),
            index,
        })
    }

    fn build_index(reader_file: &File) -> Result<HashMap<String, CommandPos>> {
        let mut index = HashMap::new();
        let mut reader = BufReader::new(reader_file);
        let mut pos = reader.seek(SeekFrom::Start(0))?;
        let mut stream = serde_json::Deserializer::from_reader(&mut reader).into_iter::<Command>();

        while let Some(cmd) = stream.next() {
            let new_pos = stream.byte_offset() as u64;
            match cmd? {
                Command::Set { key, .. } => {
                    index.insert(key, CommandPos { pos, len: new_pos - pos });
                }
                Command::Remove { key } => {
                    index.remove(&key);
                }
            }
            pos = new_pos;
        }
        Ok(index)
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

        let cmd_pos = CommandPos {
            pos,
            len: new_pos - pos,
        };
        self.index.insert(key, cmd_pos);
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
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;
            self.index.remove(&key);
            Ok(())
        } else {
            Err(KvsError::KeyNotFound)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}
