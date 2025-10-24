use std::io;

#[derive(Debug)]
pub enum KvsError {
    Io(io::Error),
    Serde(serde_json::Error),
    KeyNotFound,
    UnexpectedCommandType,
}

impl From<io::Error> for KvsError {
    fn from(err: io::Error) -> Self {
        KvsError::Io(err)
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(err: serde_json::Error) -> Self {
        KvsError::Serde(err)
    }
}

pub type Result<T> = std::result::Result<T, KvsError>;
