use crate::error::KvsError;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are stored in a `HashMap` in memory and not persisted to disk.
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
    store: HashMap<String, String>,
}

impl KvStore {
    /// Opens a `KvStore` with the given path.
    ///
    /// This will createa new directory if the given one does not exist.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let mut path = path.into();
        fs::create_dir_all(&path)?;
        path.push("wal.log");
        let wal = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&path)?;
        Ok(KvStore {
            writer: BufWriter::new(wal),
            store: Self::load_from_wal(&path)?,
        })
    }

    pub fn load_from_wal(path: impl Into<PathBuf>) -> Result<HashMap<String, String>> {
        let mut map = HashMap::new();
        let reader = BufReader::new(File::open(path.into())?);
        let stream = serde_json::Deserializer::from_reader(reader).into_iter::<Command>();
        for cmd in stream {
            match cmd? {
                Command::Set { key, value } => {
                    map.insert(key, value);
                }
                Command::Remove { key } => {
                    map.remove(&key);
                }
            }
        }
        Ok(map)
    }

    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::Set {
            key: key.clone(),
            value: value.clone(),
        };
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;
        self.store.insert(key, value);
        Ok(())
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        Ok(self.store.get(&key).cloned())
    }

    /// Remove a given key.
    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.store.contains_key(&key) {
            let cmd = Command::Remove { key: key.clone() };
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;
            self.store.remove(&key);
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
