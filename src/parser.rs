extern crate config;
extern crate toml;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::ErrorKind;
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

#[derive(Debug)]
pub enum ParserError {
    GenericError,
    ReadError,
    NotFoundError,
    EmptyFileError
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkgInfo {
    pub name:      String,
    pub version:   String,
    pub path:      String,
    pub sha256:    String,
    pub ipfs:      String,
    pub signature: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyringEntry {
    pub name:  String,
    pub email: String,
    pub key:   String
}

#[derive(Debug, Deserialize)]
struct Config {
    global_str: Option<String>,
    packages:   Option<Vec<PkgInfoInternal>>,
}

#[derive(Debug, Deserialize)]
struct PkgInfoInternal {
    name:      Option<String>,
    version:   Option<String>,
    path:      Option<String>,
    sha256:    Option<String>,
    ipfs:      Option<String>,
    signature: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
struct KeyringConfig {
    signers: Option<Vec<KeyringEntryInternal>>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyringEntryInternal {
    pub name:  Option<String>,
    pub email: Option<String>,
    pub key:   Option<String>
}

#[derive(Serialize)]
struct ConfigWriter {
    packages: Vec<PkgInfo>
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
            name:      val.name.unwrap(),
            version:   val.version.unwrap(),
            sha256:    val.sha256.unwrap(),
            path:      val.path.unwrap_or_else(|| "".to_string()),
            ipfs:      val.ipfs.unwrap_or_else(|| "".to_string()),
            signature: val.signature.unwrap_or_else(|| "".to_string())
        });
    }

    return Ok(res)
}

pub fn parsefilenew(fname: &str) -> Result<HashMap<String, PkgInfo>, ParserError> {
    let mut contents = String::new();
    let mut map: HashMap<String, PkgInfo> = HashMap::new();

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

    let config: Config = toml::from_str(&contents).unwrap();

    if !config.packages.is_some() {
        return Ok(map);
    }

    for val in config.packages.unwrap() {
        let pkgname = val.name.unwrap();
        map.insert(pkgname.clone(), PkgInfo {
            name:      pkgname.clone(),
            version:   val.version.unwrap(),
            sha256:    val.sha256.unwrap(),
            path:      val.path.unwrap_or_else(|| "".to_string()),
            ipfs:      val.ipfs.unwrap_or_else(|| "".to_string()),
            signature: val.signature.unwrap_or_else(|| "".to_string())
        });
    }

    return Ok(map)
}

pub fn updatefile(fname: &str, pkgs: Vec<PkgInfo>) {

    let path = expand(fname);

    if fs::remove_file(&path).is_err() {
        return;
    }

    let conf = ConfigWriter {
        packages: pkgs
    };

    File::create(&path)
    .unwrap()
    .write_all(
        toml::to_string(&conf)
        .unwrap()
        .as_bytes()
    ).unwrap();
}

pub fn updatefilenew(fname: &str, pkgs: HashMap<String, PkgInfo>) {

    let path = expand(fname);

    if fs::remove_file(&path).is_err() {
        return;
    }

    let mut conf = ConfigWriter {
        packages: Vec::new()
    };

    for (k, v) in pkgs.into_iter() {
        conf.packages.push(v);
    }

    File::create(&path)
    .unwrap()
    .write_all(
        toml::to_string(&conf)
        .unwrap()
        .as_bytes()
    ).unwrap();
}

pub fn parse_keyring() -> Result<Vec<String>, ParserError> {

    let mut contents = String::new();
    let mut res: Vec<String> = Vec::new();

    let mut f = match File::open(expand("KEYRING.toml")) {
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

    let config: KeyringConfig = toml::from_str(&contents).unwrap();

    if !config.signers.is_some() {
        return Err(ParserError::EmptyFileError);
    }

    for val in config.signers.unwrap() {
        res.push(val.key.unwrap());
    }

    return Ok(res)
}
