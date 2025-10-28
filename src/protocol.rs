use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Set { key: String, value: String },
    Get { key: String },
    Remove { key: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Ok(Option<String>),
    Err(String),
}
