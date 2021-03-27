extern crate base64;

use futures::{select, future, FutureExt, StreamExt, TryStreamExt};
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

use crate::parser;
use crate::ipfs;

#[actix_rt::main]
async fn handle_query(rx: mpsc::Receiver<String>) {

    let map = parser::parsefilenew(&parser::expand("pkglist_bootstrap.toml")).unwrap();
    let client = ipfs::get_client();

    loop {
        let pkg = rx.recv().unwrap();

        match map.get(&pkg) {
            Some(info) => {
                println!("package {} found!", pkg);
                client.pubsub_pub(
                    ipfs::PUBSUB_TOPIC_QURY_RESP,
                    &toml::to_string(&info).unwrap()
                ).await;
            },
            None => {
                println!("No package {} found", pkg);
            }
        }
    }
}

// create a channel which is used to communicate between this control flow
// and the thread that is responsible for answering to PUBSUB_TOPIC_QUERY
// requests. Each time a new message is received from that pubsub interface,
// the message is passed to the handle_query() fuction which holds the list
// of all available packages that the network contains.
//
// The function the responds to the query by sending serialized parser::PkgInfo
// using the PUBSUB_TOPIC_QURY_RESP interface. This way there can be multiple
// nodes in the network serving package query requests.
pub async fn daemon() {

    let (tx, rx) = mpsc::channel();

    thread::spawn(move|| { handle_query(rx) });

    let mut sub_query = {
        ipfs::get_client()
            .pubsub_sub(ipfs::PUBSUB_TOPIC_QUERY, false)
            .try_for_each(|msg| {
                tx.send(
                    std::str::from_utf8(
                        &base64::decode(msg.data.unwrap()).unwrap()
                    ).unwrap().to_owned()
                ).unwrap();
                future::ok(())
            })
            .fuse()
    };

    select! {
        res = sub_query => match res {
            Ok(_) => eprintln!("done reading messages..."),
            Err(e) => eprintln!("error reading messages: {}", e)
        },
    }
}
