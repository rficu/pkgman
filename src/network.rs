extern crate toml;
extern crate version_compare;

use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use version_compare::{CompOp, VersionCompare};

use crate::parser;
use crate::ipfs;

#[derive(Debug, Serialize, Deserialize)]
enum Commands {
    Query,
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

pub fn query(package: &str) -> Result<parser::PkgInfo, ipfs::IPFSError> {

    let mut data = [0 as u8; 1024];
    let mut ret  = parser::PkgInfo {
        name:    String::new(),
        version: String::new(),
        path:    String::new(),
        sha256:  String::new(),
        ipfs:    String::new()
    };

    match TcpStream::connect("127.0.0.1:3333") {
        Ok(mut stream) => {
            let mut request = Request {
                cmd: Commands::Query,
                info: Vec::new()
            };

            request.info.push(parser::PkgInfo {
                name:    package.to_string(),
                version: String::new(),
                path:    String::new(),
                sha256:  String::new(),
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

                    ret.name    = res.info[0].name.clone();
                    ret.version = res.info[0].version.clone();
                    ret.sha256  = res.info[0].sha256.clone();
                    ret.ipfs    = res.info[0].ipfs.clone();
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                    return Err(ipfs::IPFSError::Unknown);
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
            return Err(ipfs::IPFSError::UnableToConnect);
        }
    }

    return Ok(ret);
}

pub fn update(_config: &Vec<parser::PkgInfo>) -> Result<(), ipfs::IPFSError> {

    // TODO send a list of package names to bootstrap
    // TODO bootstrap returns a list of valid packages with their version and ipfs hash
    // TODO compare version and update all new packages
    // TODO

    Ok(())
}

pub async fn download(name: &str) -> Result<(), ipfs::IPFSError> {

    let mut pkgs = parser::parsefile(&parser::expand("pkglist.toml")).unwrap();

    match query(name) {
        Ok(pkg) => {
            // make sure we don't have the latest version of the software already
            let mut idx: usize = usize::MAX;

            for (i, our_pkg) in pkgs.iter().enumerate() {
                if our_pkg.name == pkg.name {
                    idx = i;
                    if our_pkg.version == pkg.version {
                        return Err(ipfs::IPFSError::AlreadyExists);
                    }
                }
            }

            let ret = ipfs::download(&pkg).await;

            if ret != ipfs::IPFSError::Success {
                return Err(ret);
            }

            if idx != usize::MAX {
                pkgs[0].ipfs    = pkg.ipfs.clone();
                pkgs[0].version = pkg.version.clone();
            } else {
                pkgs.push(pkg);
            }

            parser::updatefile("pkglist.toml", pkgs);
            return Ok(());
        },
        Err(err) => {
            return Err(err);
        }
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

fn handle_query(
    stream:  &mut TcpStream,
    map:     &HashMap<String, parser::PkgInfo>,
    request: &Request)
{
    let mut response = Response {
        status: ipfs::IPFSError::Success,
        info: Vec::new()
    };

    match map.get(&request.info[0].name) {
        Some(pkg) => {
            response.info.push(parser::PkgInfo {
                name:    pkg.name.clone(),
                version: pkg.version.clone(),
                sha256:  pkg.sha256.clone(),
                path:    String::new(),
                ipfs:    pkg.ipfs.clone(),
            });
        },
        None => {
            response.status = ipfs::IPFSError::NotFound
        }
    };

    stream.write(
        toml::to_string(&response)
        .unwrap()
        .as_bytes()
    ).unwrap();
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
                            Commands::Query => {
                                handle_query(&mut stream, &map, &req);
                            },
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
