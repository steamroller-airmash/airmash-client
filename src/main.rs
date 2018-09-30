extern crate airmash_client;
extern crate log;
extern crate simple_logger;

use airmash_client::Client;

use std::error::Error;
use std::time::Duration;

fn run_bot(name: &str, server: &str) -> Result<(), Box<Error>> {
    Client::new(server)?
        .login(name, "JOLLY")?
        .wait(Duration::from_secs(5))?
        .chat("TEST CHAT")?
        .wait(Duration::from_secs(2))?
        .say("-bot-")?
        .wait(Duration::from_secs(10))?
        .disconnect()?;

    Ok(())
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    if let Err(e) = run_bot("TESTBOT", "wss://game.airmash.steamroller.tk/dev") {
        println!("An error occurred:\n{}", e);
    }
}
