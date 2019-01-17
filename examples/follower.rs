//! A bot that follows a player (given by name) around
//! and leaves once that player is gone.

#![feature(futures_api, await_macro, async_await)]

extern crate airmash_client;
extern crate clap;
extern crate tokio;
extern crate url;

#[macro_use]
extern crate log;
extern crate env_logger;

use airmash_client::protocol::*;
use airmash_client::*;

use std::error::Error;

use tokio::r#await;
use url::Url;

async fn bot(
    name: String,
    server: Url,
    flag: String,
    target: String,
) -> Result<(), Box<Error + 'static>> {
    let mut client = r#await!(Client::new_insecure(server))?;

    r#await!(client.send(client::Login {
        flag,
        name,
        // This can be replaced with a session token
        // if the bot should be logged in.
        session: "none".to_owned(),

        // The server basically ignores these
        // (except to shrink the horizon).
        // 3000 is good for anything this bot will do.
        horizon_x: 3000,
        horizon_y: 3000,
        // This must always be 5
        protocol: 5,
    }))?;

    // Any packets that we send before logging in
    // will most likely be ignored by the server
    r#await!(client.wait_for_login())?;

    while let Some(_) = r#await!(client.next())? {
        let id = match client.world().names.get(&target) {
            Some(x) => *x,
            // If there is no player with that name,
            // then we'll shut down the bot.
            None => break,
        };

        // Here follow does the heavy lifting
        r#await!(client.follow(id))?;
    }

    // The name that we have on the server may not
    // be the name that was requested.
    info!("Shutting down bot {}", client.world().get_me().name);

    Ok(())
}

async fn run_bot(name: String, server: Url, flag: String, target: String) {
    match r#await!(bot(name.clone(), server, flag, target)) {
        // Tokio doesn't support directly running
        // futures with a return type other than ()
        // so we'll log the errors while shutting down.
        Err(e) => {
            error!(
                "Bot {name} ended with an error: {err}",
                name = name,
                err = e
            );
        }
        Ok(_) => (),
    }
}

fn main() {
    env_logger::init();

    let args = match parse_args() {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("{}", msg);
            return;
        }
    };

    tokio::run_async(run_bot(args.name, args.server, args.flag, args.target));
}

struct Args {
    pub target: String,
    pub server: Url,
    pub flag: String,
    pub name: String,
}

fn parse_args() -> Result<Args, String> {
    use clap::*;

    // Command line arguments
    let args = App::new("Follower Bot")
        .about("A bot that follows a player around")
        .author("STEAMROLLER")
        .arg(
            Arg::with_name("target")
                .long("target")
                .short("t")
                .help("The player name to follow around.")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("server")
                .long("server")
                .help("The server that the bot will connect to.")
                .default_value("ws://localhost:3501")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("flag")
                .long("flag")
                .short("f")
                .help("The flag that the bot will use.")
                .default_value("UN")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("name")
                .long("name")
                .short("n")
                .help("The bot's name.")
                .default_value("FOLLOWERBOT")
                .takes_value(true),
        )
        .get_matches();

    let target = match args.value_of("target") {
        Some(v) => v,
        None => {
            return Err("No target provided!".into());
        }
    };
    // Since server has a default value, there should
    // always be a value available here.
    let server = args
        .value_of("server")
        .expect("No server argument provided!");
    let flag = args.value_of("flag").expect("No flag argument provided!");
    let name = args.value_of("name").expect("No name provided!");

    let url = match server.parse() {
        Ok(url) => url,
        Err(e) => {
            return Err(format!(
                "An error occurred while parsing the server URL:\n{}",
                e
            ));
        }
    };

    Ok(Args {
        target: target.into(),
        flag: flag.into(),
        name: name.into(),
        server: url,
    })
}
