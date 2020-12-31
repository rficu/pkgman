
use std::env;

mod parser;

fn main() {

    let args: Vec<String> = env::args().collect();
    let fname = &args[1];

    // TODO command line options with explanations

    let pkg = parser::tryparse(&fname);
    println!("{}", pkg.ipfs_hash)
}
