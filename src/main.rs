extern crate airmash_client;
extern crate log;
extern crate simple_logger;

use airmash_client::Client;
use airmash_client::protocol::{Position, KeyCode};

use std::error::Error;
use std::time::Duration;

fn run_bot(name: &str, server: &str) -> Result<(), Box<Error>> {
    Client::new(server)?
        .login(name, "JOLLY")?
        .wait(Duration::from_secs(3600*24))?
        .disconnect()?;

    Ok(())
}

//const SERVER: &'static str = "wss://game.airmash.steamroller.tk/dev";
const SERVER: &'static str = "ws://localhost:3501";

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    if let Err(e) = run_bot("TESTBOT", SERVER) {
        println!("An error occurred:\n{}", e);
    }
}
