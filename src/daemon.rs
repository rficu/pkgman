extern crate base64;

use futures::{select, future, FutureExt, StreamExt, TryStreamExt};
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

use crate::parser;
use crate::ipfs;

#[actix_rt::main]
async fn handle_query(rx: mpsc::Receiver<(&'static str, String)>) {

    let map = parser::parsefilenew(&parser::expand("pkglist_bootstrap.toml")).unwrap();
    let client = ipfs::get_client();

    loop {
        let (topic, msg) = rx.recv().unwrap();

        if topic == ipfs::PUBSUB_TOPIC_QUERY {
            match map.get(&msg) {
                Some(info) => {
                    println!("package {} found!", msg);
                    client.pubsub_pub(
                        ipfs::PUBSUB_TOPIC_QURY_RESP,
                        &toml::to_string(&info).unwrap()
                    ).await;
                },
                None => {
                    println!("No package {} found", msg);
                }
            }
        } else if topic == ipfs::PUBSUB_TOPIC_KEYRING_QUERY {
            let mut contents = String::new();
            let f = File::open(parser::expand("KEYRING.toml"))
                .unwrap()
                .read_to_string(&mut contents)
                .unwrap();

            client.pubsub_pub(
                ipfs::PUBSUB_TOPIC_KEYRING,
                &contents
            ).await;
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
//
// The keyring pubsub interfaces are used to query and distribute maintainer
// information. This maintainer info contains the name, email and public key
// of all accepted mainters as well as a signature for each maintainer's
// public key which is signed by the first node of the system to prevent
// malicious third parties from distributing their own public keys
pub async fn daemon() {

    let (tx, rx) = mpsc::channel();

    thread::spawn(move|| { handle_query(rx) });

    let mut sub_query = {
        ipfs::get_client()
            .pubsub_sub(ipfs::PUBSUB_TOPIC_QUERY, false)
            .try_for_each(|msg| {
                tx.send((
                    ipfs::PUBSUB_TOPIC_QUERY,
                    std::str::from_utf8(
                        &base64::decode(msg.data.unwrap()).unwrap()
                    ).unwrap().to_owned())
                ).unwrap();
                future::ok(())
            })
            .fuse()
    };

    let mut sub_keyring = {
        ipfs::get_client()
            .pubsub_sub(ipfs::PUBSUB_TOPIC_KEYRING_QUERY, false)
            .try_for_each(|msg| {
                tx.send((
                    ipfs::PUBSUB_TOPIC_KEYRING_QUERY,
                    std::str::from_utf8(
                        &base64::decode(msg.data.unwrap()).unwrap()
                    ).unwrap().to_owned())
                ).unwrap();
                future::ok(())
            })
            .fuse()
    };

    select! {
        res = sub_keyring => match res {
            Ok(_)    => {} ,
            Err(err) => println!("Failed to read keyring query message: {}", err)
        },
        res = sub_query => match res {
            Ok(_)    => { },
            Err(err) => println!("Failed to ready package query message: {}", err)
        },
    }
}
