extern crate config;
extern crate toml;

use std::fs::File;
use std::io::prelude::*;
use std::io::ErrorKind;
use serde::Deserialize;

#[derive(Debug)]
pub enum ParserError {
    GenericError,
    ReadError,
    NotFoundError,
    EmptyFileError
}

#[derive(Debug)]
pub struct PkgInfo {
    pub name:      String, // mandatory
    pub version:   String, // mandatory
    pub ipfs_hash: String  // mandatory
}

#[derive(Debug, Deserialize)]
struct Config {
    global_str: Option<String>,
    packages:   Option<Vec<PeerConfig>>,
}

#[derive(Debug, Deserialize)]
struct PeerConfig {
    name:      Option<String>, // mandatory
    version:   Option<String>, // mandatory
    ipfs_hash: Option<String>  // mandatory
}

pub fn parsefile(fname: &str) -> Result<Vec<PkgInfo>, ParserError> {
    let mut contents = String::new();

    let mut f = match File::open(fname) {
        Ok(val)  => val,
        Err(err) => match err.kind() {
            ErrorKind::NotFound => return Err(ParserError::NotFoundError),
            _                   => return Err(ParserError::GenericError),
        }
    };

    match f.read_to_string(&mut contents) {
        Ok(_)  => (),
        Err(_) => return Err(ParserError::ReadError)
    }

    // TODO check error
    let config: Config = toml::from_str(&contents).unwrap();

    if !config.packages.is_some() {
        println!("Input file does not contain any packages!");
        return Err(ParserError::EmptyFileError);
    }

    let mut res: Vec<PkgInfo> = Vec::new();

    for val in config.packages.unwrap() {
        res.push(PkgInfo {
            name:      val.name.unwrap(),
            version:   val.ipfs_hash.unwrap(),
            ipfs_hash: val.version.unwrap()
        });
    }

    return Ok(res)
}
