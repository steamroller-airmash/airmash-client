extern crate airmash_client;
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate tokio;

use futures::Future;
use airmash_client::Client;
use airmash_client::protocol::{KeyCode, PlaneType};

use std::error::Error;
use std::time::Duration;
use tokio::prelude::future as futures;

fn run_bot(name: &str, server: &str) -> Result<(), Box<Error>> {
    let mut vals = vec![];

    for _ in 0..50 {
        let client = Client::new(server)?
            .login(name, "CA")
            .wait(Duration::from_secs(1))
            .switch_plane(PlaneType::Goliath)
            .wait(Duration::from_secs(5))
            //.enter_spectate()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(120))
            .into_boxed()
            .disconnect()
            .map(|_| ())
            .map_err(|e| { error!("An error occurred: {}", e); });
        vals.push(client);
    }

    tokio::run(futures::join_all(vals).map(|_| ()));

    Ok(())
}

const SERVER: &'static str = "wss://game.airmash.steamroller.tk/dev";
//const SERVER: &'static str = "wss://game-eu-s1.airma.sh/ctf1";//"ws://localhost:3501";

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    if let Err(e) = run_bot("BALANCEBOT", SERVER) {
        println!("An error occurred:\n{}", e);
    }
}
