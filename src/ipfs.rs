use serde::{Serialize, Deserialize};
use ipfs_api::IpfsClient;
use std::io::Write;
use futures::TryStreamExt;
use std::fs::File;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum IPFSError {
    Success,
    Unknown,
    NotFound,
    AlreadyExists,
    UnableToConnect
}

pub async fn upload(_fpath: &String) -> IPFSError {
    return IPFSError::Success;
}

pub async fn download(name: &String, ipfs_hash: &String) -> IPFSError {
    return IPFSError::Success;
}
