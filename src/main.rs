extern crate actix_rt;
extern crate config;

use std::fs;
use std::fs::File;
use std::io::Write;
use clap::{App, Arg, AppSettings};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

mod daemon;
mod ipfs;
mod network;
mod parser;

async fn update() {
    match network::update().await {
        Ok(_)    => (),
        Err(err) => println!("Error occurred: {:#?}", err)
    };
}

async fn download(name: &str) {
    match network::download(name).await {
        Ok(_) => {
            println!("Package {} downloaded!", name);
        },
        Err(err) => match err {
            ipfs::IPFSError::NotFound => {
                println!("Package {} not found on the network!", name);
                return;
            },
            ipfs::IPFSError::AlreadyExists => {
                println!("{} is up to date!", name);
                return;
            },
            _ => {
                println!("Error occurred: {:#?}", err);
                return;
            }
        }
    };
}

async fn query(name: &str) {
    let pkginfo = match network::query(name).await {
        Ok(info) => info,
        Err(err) => match err {
            ipfs::IPFSError::NotFound => {
                println!("Package {} not found on the network", name);
                return;
            },
            _ => {
                println!("Error occurred: {:#?}", err);
                return;
            }
        }
    };

    println!("name:    {}\n\
             version: {}\n\
             sha256:  {}\n\
             ipfs:    {}",
             pkginfo.name, pkginfo.version, pkginfo.sha256, pkginfo.ipfs);
}

fn init() {

    let home    = std::env::var("HOME").unwrap();
    let base    = PathBuf::from(format!("{}/.config/pkgman/", home));
    let config  = PathBuf::from(format!("{}/.config/pkgman/PKGLIST.toml", home));
    let keyring = PathBuf::from(format!("{}/.config/pkgman/KEYRING.toml", home));

    if !Path::new(&base).exists() {
        fs::create_dir(base).unwrap();
    }

    if !Path::new(&keyring).exists() {
        // TODO move this code to parser
        #[derive(Debug, Deserialize, Serialize)]
        struct KeyringWriter {
            signers: Vec<parser::KeyringEntry>
        }

        let mut vec: Vec<parser::KeyringEntry> = Vec::new();
        let init_entry = parser::KeyringEntry {
            name:  String::from("rficu"),
            email: String::from("rficu@email.com"),
            key:   String::from("3c2PgNisX4vOumXAYVETS1aDKLHYEuhKSo7i1xnwr2Y=")
        };
        vec.push(init_entry);

        let conf = KeyringWriter {
            signers: vec
        };

        File::create(&keyring)
        .unwrap()
        .write_all(
            toml::to_string(&conf)
            .unwrap()
            .as_bytes()
        ).unwrap();
    }

    if !Path::new(&config).exists() {
        File::create(&config).unwrap();
    }
}

#[actix_rt::main]
async fn main() {

    let matches = App::new("pkgman")
        .about("IPFS-based package manager for Linux")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::with_name("daemon")
                 .short("e")
                 .long("daemon")
                 .takes_value(false)
                 .help("Start pkgman as a daemon"))
        .arg(Arg::with_name("update")
                 .short("u")
                 .long("update")
                 .takes_value(false)
                 .help("Update all packages"))
        .arg(Arg::with_name("download")
                 .short("d")
                 .long("download")
                 .takes_value(true)
                 .help("Download package"))
        .arg(Arg::with_name("query")
                 .short("q")
                 .long("query")
                 .takes_value(true)
                 .help("Query package"))
        .arg(Arg::with_name("init")
                 .short("i")
                 .long("init")
                 .takes_value(false)
                 .help("Create ~/.config/pkgman/{keyring/,PKGLIST.toml} files"))
        .get_matches();

    if matches.is_present("daemon") {
        daemon::daemon().await;
    } else if matches.is_present("update") {
        update().await;
    } else if matches.is_present("download") {
        download(matches.value_of("download").unwrap()).await;
    } else if matches.is_present("init") {
        init();
    } else {
        query(matches.value_of("query").unwrap()).await;
    }
}
