extern crate config;

use std::fs::File;
use std::io::prelude::*;

pub struct PkgInfo {
    pub ipfs_hash: String, // mandatory
    pub name:      String, // mandatory
    pub ver:       String  // mandatory
}

pub fn tryparse(fname: &String) -> PkgInfo {
    let mut contents = String::new();
    let mut f = File::open(fname).expect("File not found!");
    f.read_to_string(&mut contents).expect("Failed to read file contents!");

    let mut settings = config::Config::default();

    settings.merge(config::File::with_name(fname)).unwrap();

    PkgInfo {
        ipfs_hash: settings.get::<String>("ipfs_hash").unwrap(),
        name:      settings.get::<String>("name").unwrap(),
        ver:       settings.get::<String>("version").unwrap()
    }
}
