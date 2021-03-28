extern crate ring;
extern crate untrusted;
extern crate common;
extern crate actix_rt;

use ring::signature;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use std::io::Write;
use clap::{App, Arg, AppSettings};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

use common::parser;
use common::ipfs;

fn update_keyring(keypair: &signature::Ed25519KeyPair, name: &str, email: &str, pubkey: &str) {
    let mut signers = parser::parse_keyring_entries().unwrap();
    let sig = base64::encode(keypair.sign(pubkey.as_bytes()));

    signers.push(parser::KeyringEntry {
        name:      name.to_string(),
        email:     email.to_string(),
        key:       pubkey.to_string(),
        signature: sig.to_string()
    });

    parser::update_keyring(signers);
}

async fn update_package(keypair: &signature::Ed25519KeyPair, name: &str, version: &str, path: &str) {
    let mut files = parser::parsefilenew(&parser::expand("PKGLIST_bootstrap.toml")).unwrap();

    let mut f = File::open(&path).expect("File not found");
    let metadata = fs::metadata(&path).expect("Failed to read file size");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    let mut sha256 = Sha256::new();
    sha256.update(&buffer);
    let digest = format!("{:x}", sha256.finalize());
    let sig = base64::encode(keypair.sign(digest.as_bytes()));

    match ipfs::upload(path).await {
        Ok(ipfs) => {
            files.insert(name.to_string(), parser::PkgInfo {
                name:      name.to_string(),
                version:   version.to_string(),
                sha256:    digest,
                ipfs:      ipfs,
                signature: sig.to_string()
            });

            parser::updatefilenew("PKGLIST_bootstrap.toml", files);
        },
        Err(err) => {
            println!("Failed to upload {} to IPFS", name)
        }
    }
}

fn show_usage() {
    println!("Keyring update:");
    println!("\t./pkgmain \n\
             \t\t--update-keyring\n\
             \t\t--name rficu \n\
             \t\t--email \"rficu@email.com\" \n\
             \t\t--public-key \"3c2PgNisX4vOumXAYVETS1aDKLHYEuhKSo7i1xnwr2Y=\" \n\
             \t\t--pkcs8 /home/rficu/.config/pkgman/pkcs8\n");

    println!("Package update:");
    println!("\t./pkgmain \n\
             \t\t--update-package\n\
             \t\t--name clang \n\
             \t\t--version \"11.1.0\" \n\
             \t\t--path /usr/bin/clang\n\
             \t\t--pkcs8 /home/rficu/.config/pkgman/pkcs8");
}

// read a PKCS 8-formatted key pair from a file
fn read_keypair(fpath: &str) -> signature::Ed25519KeyPair {
    let mut f = File::open(&fpath).expect("File not found");
    let metadata = fs::metadata(&fpath).expect("Failed to read file size");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    return signature::Ed25519KeyPair::from_pkcs8(&buffer).unwrap();
}

#[actix_rt::main]
async fn main() {

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
        .arg(Arg::with_name("usage")
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

    if matches.is_present("update-package") {
        update_package(
            &key_pair,
            matches.value_of("name").unwrap(),
            matches.value_of("version").unwrap(),
            matches.value_of("path").unwrap()
        ).await;

        return;
    }
}
