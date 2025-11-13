use crate::{KvsEngine, KvsError, Result};
use sled::Db;
use std::path::PathBuf;

/// A key-value store using the `sled` storage engine.
#[derive(Clone)]
pub struct SledKvsEngine(Db);

impl SledKvsEngine {
    /// Opens a `SledKvsEngine` with the given path.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let db = sled::open(path.into())?;
        Ok(SledKvsEngine(db))
    }
}

impl KvsEngine for SledKvsEngine {
    /// Sets the value of a string key to a string.
    fn set(&self, key: String, value: String) -> Result<()> {
        self.0.insert(key, value.as_bytes())?;
        self.0.flush()?;
        Ok(())
    }

    /// Gets the string value of a given string key.
    fn get(&self, key: String) -> Result<Option<String>> {
        let value = self.0
            .get(key)?
            .map(|ivec| String::from_utf8(ivec.to_vec())).transpose()?;
        Ok(value)
    }

    /// Removes a given key.
    fn remove(&self, key: String) -> Result<()> {
        self.0.remove(key)?.ok_or(KvsError::KeyNotFound)?;
        self.0.flush()?;
        Ok(())
    }
}
