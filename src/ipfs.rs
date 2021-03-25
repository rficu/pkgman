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

pub async fn download(pkg: &parser::PkgInfo) -> IPFSError {
    let client = IpfsClient::default();

    match client
        .cat(&pkg.ipfs)
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await
    {
        Ok(res) => {
            File::create(format!("{}/{}", "packages", pkg.name))
                .unwrap()
                .write_all(&res)
                .unwrap();
        }
        Err(e) => println!("error getting file: {}", e)
    }

    return IPFSError::Success;
}
