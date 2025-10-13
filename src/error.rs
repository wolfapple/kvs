#[derive(Debug)]
pub enum KvsError {}

pub type Result<T> = std::result::Result<T, KvsError>;