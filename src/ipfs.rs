use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum IPFSError {
    Success,
    Unknown,
    NotFound,
    UnableToConnect
}

