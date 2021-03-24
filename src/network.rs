extern crate toml;

use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};

use crate::parser;

#[derive(Debug, Serialize, Deserialize)]
enum Commands {
    Query,
    Add,
    Upload,
    Download
}

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    cmd:  Commands,
    info: Vec<parser::PkgInfo>
}

pub fn update(_config: &Vec<parser::PkgInfo>) -> Result<(), IPFSError> {
    Ok(())
}

pub fn download(_name: &str) -> Result<(), IPFSError> {
    Ok(())
}

fn add_pkg(_package: &parser::PkgInfo) -> Result<(), IPFSError> {
    Ok(())
}

pub fn add(pkgs: &Vec<parser::PkgInfo>) -> Result<(), IPFSError> {

    for pkg in pkgs {
        let res = add_pkg(pkg);

        if res.is_err() {
            return Err(res.err().unwrap());
        }
    }

    Ok(())
}

pub fn bootstrap() {

    // construct a hasmap of all the packages that are available on the network
    let home  = std::env::var("HOME").unwrap();
    let fname = PathBuf::from(format!("{}/.config/pkgman/pkglist_bootstrap.toml", home));
    let mut map: HashMap<&String, &parser::PkgInfo> = HashMap::new();

    if !Path::new(&fname).exists() {
        println!("Config file not found!");
        return;
    }

    let pkgs = parser::parsefile(&fname.into_os_string().into_string().unwrap()).unwrap();

    for (_, e) in pkgs.iter().enumerate() {
        map.insert(&e.name, e);
    }

    let listener = TcpListener::bind("127.0.0.1:3333").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {

                let mut data = [0 as u8; 1024];

                match stream.read(&mut data) {
                    Ok(size) => {
                        let req: Request = toml::from_str(
                            std::str::from_utf8(&data[0..size]).unwrap()
                        ).unwrap();

                        match req.cmd {
                            Commands::Query => { },
                            Commands::Add => { },
                            Commands::Upload => { },
                            Commands::Download => { }
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
