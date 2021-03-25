use serde::{Serialize, Deserialize};
use ipfs_api::IpfsClient;
use std::io::Write;
use futures::TryStreamExt;
use std::fs::File;
use crate::parser;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum IPFSError {
    Success,
    Unknown,
    NotFound,
    AlreadyExists,
    UnableToConnect,
    NewerExists
}

pub async fn upload(pkg: &parser::PkgInfo) -> Result<String, IPFSError> {

    let client = IpfsClient::default();
    let file = File::open(&pkg.path).unwrap();

    match client.add(file).await {
        Ok(file) => return Ok(file.hash),
        Err(err) => {
            println!("Failed to add file: {:#?}", err);
            return Err(IPFSError::Unknown);
        }
    }
}

pub async fn download(name: &String, ipfs_hash: &String) -> IPFSError {
    return IPFSError::Success;
}
