extern crate toml;
extern crate ring;
extern crate untrusted;
extern crate version_compare;

use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use version_compare::{CompOp, VersionCompare};
use futures::{select, future, FutureExt, StreamExt, TryStreamExt};
use std::time::Duration;
use tokio::time::*;
use ring::signature;

use crate::parser;
use crate::ipfs;

pub async fn query(pkg: &str) -> Result<parser::PkgInfo, ipfs::IPFSError> {

    ipfs::get_client().pubsub_pub(ipfs::PUBSUB_TOPIC_QUERY, pkg).await.unwrap();

    loop {
        match tokio::time::timeout(
            Duration::from_secs(3),
            ipfs::get_client().pubsub_sub(ipfs::PUBSUB_TOPIC_QURY_RESP, false).next()).await
        {
            Ok(response) => {
                match response {
                    Some(msg) => {
                        let ret: parser::PkgInfo = toml::from_str(
                            std::str::from_utf8(
                                &base64::decode(msg.unwrap().data.unwrap()).unwrap()
                            ).unwrap()
                        ).unwrap();

                        if ret.name == pkg {
                            return Ok(ret);
                        }
                    },
                    None => {
                        println!("None");
                        return Err(ipfs::IPFSError::NotFound);
                    }
                }
            },
            Err(_err) => {
                return Err(ipfs::IPFSError::NotFound);
            }
        }
    }
}

pub async fn update() -> Result<(), ipfs::IPFSError> {

    for pkg in parser::parsefile(&parser::expand("PKGLIST.toml")).unwrap() {
        match download(&pkg.name).await {
            Ok(_) => {
                println!("Package {} updated successfully!", pkg.name);
            },
            Err(err) => {
                println!("Failed to update package {}: {:#?}", pkg.name, err);
            }
        }
    }

    Ok(())
}

pub async fn download(name: &str) -> Result<(), ipfs::IPFSError> {

    let mut pkgs = parser::parsefilenew(&parser::expand("PKGLIST.toml")).unwrap();

    match query(name).await {
        Ok(pkg) => {
            let mut new_pkg = pkg.clone();

            match pkgs.get(name) {
                Some(our_pkg) => {
                    if pkg.version == our_pkg.version {
                        return Err(ipfs::IPFSError::AlreadyExists);
                    }
                },
                None => { }
            }

            match ipfs::download(&pkg).await {
                Ok(_) => {
                    pkgs.insert(pkg.name, new_pkg);
                    parser::updatefilenew("PKGLIST.toml", pkgs);
                    return Ok(());
                },
                Err(err) => return Err(err)
            }
        },
        Err(err) => return Err(err)
    }
}

pub async fn update_keyring() -> Result<(), ipfs::IPFSError> {

    ipfs::get_client().pubsub_pub(ipfs::PUBSUB_TOPIC_KEYRING_QUERY, "update").await.unwrap();

    loop {
        match tokio::time::timeout(
            Duration::from_secs(3),
            ipfs::get_client().pubsub_sub(ipfs::PUBSUB_TOPIC_KEYRING, false).next()).await
        {
            Ok(response) => {
                match response {
                    Some(msg) => {
                        let trusted = base64::decode("3c2PgNisX4vOumXAYVETS1aDKLHYEuhKSo7i1xnwr2Y=").unwrap();
                        let mut accepted: Vec<parser::KeyringEntry> = Vec::new();

                        let signers: parser::KeyringConfig = toml::from_str(
                            std::str::from_utf8(
                                &base64::decode(msg.unwrap().data.unwrap()).unwrap()
                            ).unwrap()
                        ).unwrap();

                        for signer in signers.signers {
                            let pbkey = signature::UnparsedPublicKey::new(&signature::ED25519, &trusted);
                            let sig = base64::decode(&signer.signature).unwrap();

                            match pbkey.verify(&signer.key.as_bytes(), sig.as_ref()) {
                                Ok(_) => {
                                    println!("{} ({}) accepted!", signer.name, signer.email);
                                    accepted.push(signer);
                                },
                                Err(err) => {
                                    println!("{} ({}) rejected!", signer.name, signer.email);
                                }
                            }
                        }

                        if accepted.len() == 0 {
                            return Err(ipfs::IPFSError::SignatureMismatch);
                        }

                        parser::update_keyring(accepted);
                        return Ok(());
                    },
                    None => {
                        println!("None");
                        return Err(ipfs::IPFSError::NotFound);
                    }
                }
            },
            Err(_err) => {
                return Err(ipfs::IPFSError::NotFound);
            }
        }
    }
}
