use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvsError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Key not found")]
    KeyNotFound,
    #[error("Unexpected command type")]
    UnexpectedCommandType,
    #[error("{0}")]
    StringError(String),
}

pub type Result<T> = std::result::Result<T, KvsError>;
