use serde::{Serialize, Deserialize};
use ipfs_api::IpfsClient;
use std::io::Write;
use futures::TryStreamExt;
use std::fs::File;
use crate::parser;
use sha2::{Sha256, Digest};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum IPFSError {
    Success,
    Unknown,
    NotFound,
    AlreadyExists,
    UnableToConnect,
    NewerExists,
    ChecksumMistmatch
}

pub static PUBSUB_TOPIC_QUERY:     &'static str = "pkgman_sub_query";
pub static PUBSUB_TOPIC_QURY_RESP: &'static str = "pkgman_sub_query_response";

pub fn get_client() -> IpfsClient {
    return IpfsClient::default();
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
            let mut sha256 = Sha256::new();
            sha256.update(&res);

            if pkg.sha256 != format!("{:x}", sha256.finalize()) {
                return IPFSError::ChecksumMistmatch;
            }

            File::create(format!("{}/{}", "packages", pkg.name))
                .unwrap()
                .write_all(&res)
                .unwrap();
        }
        Err(e) => println!("error getting file: {}", e)
    }

    return IPFSError::Success;
}
