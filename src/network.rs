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
    Upload,
    Update
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

pub async fn update(config: &Vec<parser::PkgInfo>) -> Result<(), ipfs::IPFSError> {

    let mut data = [0 as u8; 1024];

    match TcpStream::connect("127.0.0.1:3333") {
        Ok(mut stream) => {
            let mut request = Request {
                cmd: Commands::Update,
                info: config.to_vec()
            };

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

                    for pkg in res.info {
                        download(&pkg.name).await;
                    }
                },
                Err(err) => {
                    println!("Error occurred: {}", err);
                }
            }
        },
        Err(err) => {
            println!("Error occurred: {}", err);
        }
    }

    Ok(())
}

pub async fn download(name: &str) -> Result<(), ipfs::IPFSError> {

    let mut pkgs = parser::parsefilenew(&parser::expand("pkglist.toml")).unwrap();

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
                    parser::updatefilenew("pkglist.toml", pkgs);
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

    let mut own_pkgs = parser::parsefile(&parser::expand("pkglist.toml")).unwrap();

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

fn handle_upload(
    stream:  &mut TcpStream,
    map:     &mut HashMap<String, parser::PkgInfo>,
    request: &mut Request)
{
    let mut response = Response {
        status: ipfs::IPFSError::Success,
        info: Vec::new()
    };

    match map.get(&request.info[0].name) {
        Some(pkg) => {
            // package already exists in the network
            //
            // compare version numbers, if the version of the received package
            // is equal to or smaller than the version of the package that the network
            // has, the package is rejected
            //
            // if the package is newer, it is accepted
            match VersionCompare::compare(&pkg.version, &request.info[0].version).unwrap() {
                CompOp::Lt => response.status = ipfs::IPFSError::Success,
                CompOp::Eq => response.status = ipfs::IPFSError::AlreadyExists,
                CompOp::Gt => response.status = ipfs::IPFSError::NewerExists,
                _ => unreachable!()
            }
        },
        None => {
            // new package
        }
    };

    stream.write(
        toml::to_string(&response)
        .unwrap()
        .as_bytes()
    ).unwrap();

    if response.status == ipfs::IPFSError::Success {
        let mut ipfs = [0 as u8; 64];

        match stream.read(&mut ipfs) {
            Ok(size) => {
                // remove the old version of the package from the hashmap
                // and add a new entry with updated/new fields
                request.info[0].ipfs = std::str::from_utf8(&ipfs[0..size]).unwrap().to_string();
                map.remove(&request.info[0].name);
                map.insert(request.info[0].name.clone(), request.info[0].clone());
                println!("Package {} added or updated to version {}", request.info[0].name, request.info[0].version);
            },
            Err(err) => {
                println!("Failed to receive an IPFS hash from the client: {:#?}", err);
            }
        }
    }
}

fn handle_update(
    stream:  &mut TcpStream,
    map:     &HashMap<String, parser::PkgInfo>,
    request: &Request)
{
    let mut response = Response {
        status: ipfs::IPFSError::Success,
        info: Vec::new()
    };

    for pkg in &request.info {
        match map.get(&pkg.name) {
            Some(found_pkg) => {
                println!("package found on the server");

                match VersionCompare::compare(&found_pkg.version, &pkg.version).unwrap() {
                    CompOp::Gt => response.info.push(pkg.clone()),
                    _ => {
                        println!("package up to date");
                    }
                }
            },
            None => {
                println!("Client has an unregistered package!");
            }
        }
    }

    stream.write(
        toml::to_string(&response)
        .unwrap()
        .as_bytes()
    ).unwrap();
}

pub fn bootstrap() {

    // construct a hasmap of all the packages that are available on the network
    let mut map: HashMap<String, parser::PkgInfo> = HashMap::new();

    let listener = TcpListener::bind("127.0.0.1:3333").unwrap();
    let pkgs     = parser::parsefile(&parser::expand("pkglist_bootstrap.toml")).unwrap();

    for (_, e) in pkgs.iter().enumerate() {
        map.insert(e.name.clone(), e.clone());
    }

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {

                let mut data = [0 as u8; 1024];

                match stream.read(&mut data) {
                    Ok(size) => {
                        let mut req: Request = toml::from_str(
                            std::str::from_utf8(&data[0..size]).unwrap()
                        ).unwrap();

                        match req.cmd {
                            Commands::Upload => {
                                handle_upload(&mut stream, &mut map, &mut req);
                            },
                            Commands::Update => {
                                handle_update(&mut stream, &map, &req);
                            }
                        }
                    },
                    Err(_) => {
                        println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
                        stream.shutdown(Shutdown::Both).unwrap();
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    drop(listener);
}
