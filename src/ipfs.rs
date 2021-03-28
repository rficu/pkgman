extern crate ring;
extern crate untrusted;

use serde::{Serialize, Deserialize};
use ipfs_api::IpfsClient;
use std::io::Write;
use futures::TryStreamExt;
use std::fs::File;
use crate::parser;
use sha2::{Sha256, Digest};
use ring::signature;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum IPFSError {
    Success,
    Unknown,
    NotFound,
    AlreadyExists,
    UnableToConnect,
    NewerExists,
    ChecksumMismatch,
    SignatureMismatch
}

// Publish/Subscribe Topics (PST)
pub static PST_PACKAGE:       &'static str = "pkgman_sub_query";
pub static PST_PACKAGE_QUERY: &'static str = "pkgman_sub_query_response";
pub static PST_KEYRING:       &'static str = "pkgman_sub_keyring_query";
pub static PST_KEYRING_QUERY: &'static str = "pkgman_sub_keyring";

pub fn get_client() -> IpfsClient {
    return IpfsClient::default();
}

pub async fn download(pkg: &parser::PkgInfo) -> Result<(), IPFSError> {
    let client = IpfsClient::default();

    match client
        .cat(&pkg.ipfs)
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await
    {
        Ok(res) => {
            // before the package is installed to the system
            // verify that its sha256 checksum and signatures are valid
            let mut sha256 = Sha256::new();
            sha256.update(&res);
            let digest = sha256.finalize();
            let sig = base64::decode(&pkg.signature).unwrap();

            if pkg.sha256 != format!("{:x}", digest) {
                return Err(IPFSError::ChecksumMismatch);
            }

            for key in parser::parse_keyring().unwrap() {
                let pbkey = signature::UnparsedPublicKey::new(
                    &signature::ED25519,
                    base64::decode(&key).unwrap()
                );

                match pbkey.verify(&pkg.sha256.as_bytes(), sig.as_ref()) {
                    Ok(_) => {
                        File::create(format!("{}/{}", "packages", pkg.name))
                            .unwrap()
                            .write_all(&res)
                            .unwrap();

                        return Ok(())
                    },
                    // do not exit here as there may be multiple keys of which only one will
                    // provide the correct checksum. Error is returned after the loop
                    Err(_err) => { }
                }
            }

            println!("Failed to verify signature!");
            return Err(IPFSError::SignatureMismatch);
        }
        Err(e) => {
            println!("error getting file: {}", e);
            return Err(IPFSError::Unknown);
        }
    }
}
