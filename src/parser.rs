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
    pub sha256:    String,
    pub ipfs:      String,
    pub signature: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyringConfig {
    pub signers: Vec<KeyringEntry>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyringEntry {
    pub name:      String,
    pub email:     String,
    pub key:       String,
    pub signature: String
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
    sha256:    Option<String>,
    ipfs:      Option<String>,
    signature: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
struct KeyringConfigInternal {
    signers: Option<Vec<KeyringEntryInternal>>
}

#[derive(Debug, Deserialize, Serialize)]
struct KeyringEntryInternal {
    name:      Option<String>,
    email:     Option<String>,
    key:       Option<String>,
    signature: Option<String>
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
            ipfs:      val.ipfs.unwrap_or_else(|| "".to_string()),
            signature: val.signature.unwrap_or_else(|| "".to_string())
        });
    }

    return Ok(map)
}

pub fn updatefilenew(fname: &str, pkgs: HashMap<String, PkgInfo>) {

    let path = expand(fname);

    if fs::remove_file(&path).is_err() {
        return;
    }

    let mut conf = ConfigWriter {
        packages: Vec::new()
    };

    for (_, v) in pkgs.into_iter() {
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

    let config: KeyringConfigInternal = toml::from_str(&contents).unwrap();

    if !config.signers.is_some() {
        return Err(ParserError::EmptyFileError);
    }

    for val in config.signers.unwrap() {
        res.push(val.key.unwrap());
    }

    return Ok(res)
}

pub fn parse_keyring_entries() -> Result<Vec<KeyringEntry>, ParserError> {

    let mut contents = String::new();
    let mut res: Vec<KeyringEntry> = Vec::new();

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

    let config: KeyringConfigInternal = toml::from_str(&contents).unwrap();

    if !config.signers.is_some() {
        return Err(ParserError::EmptyFileError);
    }

    for val in config.signers.unwrap() {
        res.push(KeyringEntry {
            name:      val.name.unwrap().clone(),
            email:     val.email.unwrap().clone(),
            key:       val.key.unwrap().clone(),
            signature: val.signature.unwrap().clone()
        });
    }

    return Ok(res)
}

fn update_keyring_internal(signers: Vec<KeyringEntry>) {

    let path = expand("KEYRING.toml");

    if fs::remove_file(&path).is_err() {
        return;
    }

    let conf = KeyringConfig {
        signers: signers
    };

    File::create(&path)
    .unwrap()
    .write_all(
        toml::to_string(&conf)
        .unwrap()
        .as_bytes()
    ).unwrap();
}

pub fn update_keyring(signers: Vec<KeyringEntry>) {
    update_keyring_internal(signers);
}

pub fn update_keyring_default() {

    let mut vec: Vec<KeyringEntry> = Vec::new();
    let init_entry = KeyringEntry {
        name:      String::from("rficu"),
        email:     String::from("rficu@email.com"),
        key:       String::from("3c2PgNisX4vOumXAYVETS1aDKLHYEuhKSo7i1xnwr2Y="),
        signature: String::from("+Bl5DtPMfKsxKd4eNQybgpbcrF70TuyMfp3Eyu8xQ1CWkBrhDEcx0jUO084EMZ7dbVw/v+0x0MMkbX/gZlGvBQ==")

    };

    vec.push(init_entry);
    update_keyring_internal(vec);
}

pub fn get_file_contents(path: &str) -> Vec<u8> {
    let mut f = File::open(&path).expect("File not found");
    let metadata = fs::metadata(&path).expect("Failed to read file size");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    return buffer;
}
