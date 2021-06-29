//! A bot that follows a player (given by name) around
//! and leaves once that player is gone.

#![feature(futures_api, async_await)]

extern crate airmash_client;
extern crate clap;
extern crate tokio;
extern crate url;

#[macro_use]
extern crate log;
extern crate env_logger;

use airmash_client::protocol::*;
use airmash_client::*;

use std::env;
use std::error::Error;
use std::time::{Duration, Instant};

use url::Url;

async fn bot(
    name: String, 
    server: Url,
    flag: FlagCode,
) -> Result<(), Box<dyn Error + 'static>> {
    let mut client = Client::new(server).await?;

    client.send(client::Login {
        flag: <&str>::from(flag).into(),//unimplemented!(),//flag.to_string(),
        name: name.into(),
        session: "none".into(),

        // The server basically ignores these
        // (except to shrink the horizon).
        // 3000 is good for anything this bot will do.
        horizon_x: 3000,
        horizon_y: 3000,
        // This must always be 5
        protocol: 5,
    }).await?;

    Ok(())
}

fn main() {

}
