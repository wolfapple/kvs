use crate::{KvsEngine, KvsError, Result};
use sled::Db;
use std::path::PathBuf;

/// A key-value store using the `sled` storage engine.
#[derive(Clone)]
pub struct SledKvsEngine {
    db: Db,
}

impl SledKvsEngine {
    /// Opens a `SledKvsEngine` with the given path.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let db = sled::open(path.into())?;
        Ok(SledKvsEngine { db })
    }
}

impl KvsEngine for SledKvsEngine {
    /// Sets the value of a string key to a string.
    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.db.insert(key, value.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    /// Gets the string value of a given string key.
    fn get(&mut self, key: String) -> Result<Option<String>> {
        let value = self.db
            .get(key)?
            .map(|ivec| String::from_utf8(ivec.to_vec())).transpose()?;
        Ok(value)
    }

    /// Removes a given key.
    fn remove(&mut self, key: String) -> Result<()> {
        self.db.remove(key)?.ok_or(KvsError::KeyNotFound)?;
        self.db.flush()?;
        Ok(())
    }
}
