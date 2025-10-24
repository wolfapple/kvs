#[derive(Debug)]
pub enum KvsError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    KeyNotFound,
}

impl From<std::io::Error> for KvsError {
    fn from(e: std::io::Error) -> Self {
        KvsError::Io(e)
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(e: serde_json::Error) -> Self {
        KvsError::Serde(e)
    }
}

pub type Result<T> = std::result::Result<T, KvsError>;