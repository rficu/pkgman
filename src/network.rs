extern crate toml;
extern crate version_compare;

use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use version_compare::{CompOp, VersionCompare};
use futures::{select, future, FutureExt, StreamExt, TryStreamExt};
use std::time::Duration;
use tokio::time::*;

use crate::parser;
use crate::ipfs;

#[derive(Debug, Serialize, Deserialize)]
enum Commands {
    Upload
}

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    cmd:  Commands,
    info: Vec<parser::PkgInfo>
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    status: ipfs::IPFSError,
    info: Vec<parser::PkgInfo>
}

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

async fn upload_pkg(pkg: &parser::PkgInfo) -> Result<String, ipfs::IPFSError> {

    let mut data = [0 as u8; 1024];

    match TcpStream::connect("127.0.0.1:3333") {
        Ok(mut stream) => {
            let mut request = Request {
                cmd: Commands::Upload,
                info: Vec::new()
            };

            request.info.push(parser::PkgInfo {
                name:    pkg.name.clone(),
                version: pkg.version.clone(),
                path:    String::new(),
                sha256:  pkg.sha256.clone(),
                ipfs:    String::new()
            });

            stream.write(
                toml::to_string(&request)
                .unwrap()
                .as_bytes()
            ).unwrap();

            match stream.read(&mut data) {
                Ok(size) => {
                    let res: Response = toml::from_str(
                        std::str::from_utf8(&data[0..size]).unwrap()
                    ).unwrap();

                    if res.status != ipfs::IPFSError::Success {
                        return Err(res.status);
                    }

                    match ipfs::upload(&pkg).await {
                        Ok(ipfs) => {
                            stream.write(&ipfs.as_bytes()).unwrap();
                            return Ok(ipfs);
                        },
                        Err(err) => {
                            return Err(err);
                        }
                    }
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                    return Err(ipfs::IPFSError::Unknown);
                }
            }
        },
        Err(_) => return Err(ipfs::IPFSError::UnableToConnect)
    }
}

pub async fn add(pkgs: &mut Vec<parser::PkgInfo>) -> Result<(), ipfs::IPFSError> {

    let mut own_pkgs = parser::parsefile(&parser::expand("PKGLIST.toml")).unwrap();

    for pkg in pkgs {
        match upload_pkg(pkg).await {
            Ok(ipfs) => {
                // TODO remove "pkg.name" from "own_pkgs" (hashmap?)
                own_pkgs.push(parser::PkgInfo {
                    name:    pkg.name.clone(),
                    version: pkg.version.clone(),
                    path:    String::new(),
                    sha256:  pkg.sha256.clone(),
                    ipfs:    ipfs.clone()
                });
            },
            Err(err) => {
                match err {
                    ipfs::IPFSError::AlreadyExists => println!("{} up to date with server's version!", pkg.name),
                    ipfs::IPFSError::NewerExists => println!("{} is older than server's version!", pkg.name),
                    _ => println!("{:#?}", err)
                }
            }
        }
    }

    parser::updatefile("pkglist.toml", own_pkgs);
    Ok(())
}
