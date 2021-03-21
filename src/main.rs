use std::path::{Path, PathBuf};
use clap::{App, Arg, AppSettings};

mod parser;
mod network;

fn update() {
    let home  = std::env::var("HOME").unwrap();
    let fname = PathBuf::from(format!("{}/.config/pkgman/pkglist.toml", home));

    if !Path::new(&fname).exists() {
        println!("Config file not found!");
        return;
    }

    let path = &fname.into_os_string().into_string().unwrap();

    match network::add(&parser::parseconfig(path).unwrap()) {
        Ok(_)    => (),
        Err(err) => println!("Error occurred: {:#?}", err)
    };
}

fn add(fname: &str) {
    match network::add(&parser::parsefile(fname).unwrap()) {
        Ok(_)    => (),
        Err(err) => println!("Error occurred: {:#?}", err)
    };
}

fn download(name: &str) {
    match network::download(name) {
        Ok(_)    => (),
        Err(err) => println!("Error occurred: {:#?}", err)
    };
}

fn query(name: &str) {
    let pkg_info = match network::query(name) {
        Ok(info) => info,
        Err(err) => {
            println!("Error occurred: {:#?}", err);
            return ();
        }
    };

    println!("{:#?}", pkg_info);
}

fn main() {

    let matches = App::new("pkgman")
        .about("IPFS-based package manager for Linux")
        .setting(AppSettings::ArgRequiredElseHelp)
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

    if matches.is_present("update") {
        update();
    } else if matches.is_present("add") {
        add(matches.value_of("add").unwrap());
    } else if matches.is_present("download") {
        download(matches.value_of("download").unwrap());
    } else {
        query(matches.value_of("query").unwrap());
    }
}
