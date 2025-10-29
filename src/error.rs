use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvsError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Sled error: {0}")]
    Sled(#[from] sled::Error),
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("Key not found")]
    KeyNotFound,
    #[error("Unexpected command type")]
    UnexpectedCommandType,
    #[error("Engine mismatch")]
    EngineMismatch,
    #[error("{0}")]
    StringError(String),
}

pub type Result<T> = std::result::Result<T, KvsError>;
