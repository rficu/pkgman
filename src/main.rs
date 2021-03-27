extern crate actix_rt;

use clap::{App, Arg, AppSettings};

mod daemon;
mod ipfs;
mod network;
mod parser;

async fn update() {
    match network::update(&parser::parsefile(&parser::expand("pkglist.toml")).unwrap()).await {
        Ok(_)    => (),
        Err(err) => println!("Error occurred: {:#?}", err)
    };
}

async fn add(fname: &str) {
    match network::add(&mut parser::parsefile(fname).unwrap()).await {
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

fn query(name: &str) {
    let pkginfo = match network::query(name) {
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
        .arg(Arg::with_name("add")
                 .short("a")
                 .long("add")
                 .takes_value(true)
                 .help("Add package"))
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
        .get_matches();

    if matches.is_present("daemon") {
        daemon::daemon().await;
    } else if matches.is_present("update") {
        update().await;
    } else if matches.is_present("add") {
        add(matches.value_of("add").unwrap()).await;
    } else if matches.is_present("download") {
        download(matches.value_of("download").unwrap()).await;
    } else {
        query(matches.value_of("query").unwrap());
    }
}
