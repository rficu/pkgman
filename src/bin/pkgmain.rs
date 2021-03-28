extern crate ring;
extern crate untrusted;
extern crate common;

use ring::signature;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use std::io::Write;
use clap::{App, Arg, AppSettings};
use serde::{Serialize, Deserialize};

use common::parser;
use common::ipfs;

fn update_keyring(keypair: &signature::Ed25519KeyPair, name: &str, email: &str, pubkey: &str) {
    // TODO
}

fn update_package(keypair: &signature::Ed25519KeyPair, name: &str, version: &str, path: &str) {
    // TODO
}

fn show_usage() {
    // TODO
}

// read a PKCS 8-formatted key pair from a file
fn read_keypair(fpath: &str) -> signature::Ed25519KeyPair {
    let mut f = File::open(&fpath).expect("File not found");
    let metadata = fs::metadata(&fpath).expect("Failed to read file size");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    return signature::Ed25519KeyPair::from_pkcs8(&buffer).unwrap();
}

fn main() {

    let matches = App::new("")
        .about("Maintainer tool for pgkman")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::with_name("update-keyring")
                 .short("k")
                 .long("update-keyring")
                 .takes_value(false)
                 .help("Update keyring, mutually exclusive with --update-package"))
        .arg(Arg::with_name("update-package")
                 .short("p")
                 .long("update-package")
                 .takes_value(false)
                 .help("Update package list, mutually exclusive with --update-keyring"))
        .arg(Arg::with_name("email")
                 .long("email")
                 .takes_value(true)
                 .help("Email of the maintainer"))
        .arg(Arg::with_name("public-key")
                 .long("public-key")
                 .takes_value(true)
                 .help("Base64-encode public key of the maintainer"))
        .arg(Arg::with_name("name")
                 .long("name")
                 .takes_value(true)
                 .help("Name of the maintainer/package"))
        .arg(Arg::with_name("version")
                 .long("version")
                 .takes_value(true)
                 .help("Version number of the package"))
        .arg(Arg::with_name("path")
                 .long("path")
                 .takes_value(true)
                 .help("Path to the binary of the package"))
        .arg(Arg::with_name("pkcs8")
                 .long("pkcs8")
                 .takes_value(true)
                 .help("Full path to the PKCS 8-formatted keypair"))
        .arg(Arg::with_name("usages")
                 .short("u")
                 .long("usage")
                 .takes_value(false)
                 .help("Show usage for the tool"))
        .get_matches();

    if (matches.is_present("usage")) {
        show_usage();
        return;
    }

    let key_pair = read_keypair(matches.value_of("pkcs8").unwrap());

    if matches.is_present("update-keyring") {
        update_keyring(
            &key_pair,
            matches.value_of("name").unwrap(),
            matches.value_of("email").unwrap(),
            matches.value_of("public-key").unwrap()
        );
        return;
    }

    if matches.is_present("update package") {
        update_package(
            &key_pair,
            matches.value_of("name").unwrap(),
            matches.value_of("version").unwrap(),
            matches.value_of("path").unwrap()
        );

        return;
    }
}
