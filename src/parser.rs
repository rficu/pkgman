extern crate config;
extern crate toml;

use std::fs::File;
use std::io::prelude::*;
use std::io::ErrorKind;
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum ParserError {
    GenericError,
    ReadError,
    NotFoundError,
    EmptyFileError
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PkgInfo {
    pub name:    String,
    pub version: String,
    pub path:    String,
    pub sha256:  String,
    pub ipfs:    String
}

#[derive(Debug, Deserialize)]
struct Config {
    global_str: Option<String>,
    packages:   Option<Vec<PkgInfoInternal>>,
}

#[derive(Debug, Deserialize)]
struct PkgInfoInternal {
    name:    Option<String>,
    version: Option<String>,
    path:    Option<String>,
    sha256:  Option<String>,
    ipfs:    Option<String>
}

pub fn expand(config: &str) -> String {
    let home  = std::env::var("HOME").unwrap();
    let fname = PathBuf::from(format!("{}/.config/pkgman/{}", home, config));

    if !Path::new(&fname).exists() {
        println!("Config file not found!");
        return String::new();
    }

    return fname.into_os_string().into_string().unwrap();
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
            name:    val.name.unwrap(),
            version: val.version.unwrap(),
            sha256:  val.sha256.unwrap(),
            path:    val.path.unwrap(),
            ipfs:    val.ipfs.unwrap()
        });
    }

    return Ok(res)
}
