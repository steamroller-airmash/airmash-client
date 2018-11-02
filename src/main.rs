extern crate airmash_client;
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate tokio;

use airmash_client::protocol::{KeyCode, PlaneType};
use airmash_client::ClientStream;
use futures::Future;

use std::env;
use std::error::Error;
use std::time::Duration;
use tokio::prelude::future as futures;

fn run_bot(name: &str, server: &str) -> Result<(), Box<Error>> {
    env::set_var("RUST_BACKTRACE", "1");

    let mut vals = vec![];

    for i in 0..1 {
        let client = ClientStream::new(server)?
            .login(&format!("{} {}", name, i), "CA")
            .wait(Duration::from_secs(1))
            .switch_plane(PlaneType::Goliath)
            .wait(Duration::from_secs(5))
            //.enter_spectate()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .into_boxed()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .into_boxed()
            .wait(Duration::from_secs(15))
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .into_boxed()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .into_boxed()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .into_boxed()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .into_boxed()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .into_boxed()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .into_boxed()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .into_boxed()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .into_boxed()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .into_boxed()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .into_boxed()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .into_boxed()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .into_boxed()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .into_boxed()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .into_boxed()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
            .into_boxed()
            .press_key(KeyCode::Up)
            .press_key(KeyCode::Fire)
            .wait(Duration::from_secs(15))
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

    if let Err(e) = run_bot("DEATHBOT", SERVER) {
        println!("An error occurred:\n{}", e);
    }
}
