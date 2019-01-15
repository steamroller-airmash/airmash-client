#![feature(futures_api, await_macro, async_await)]
#![allow(dead_code)]

extern crate airmash_client;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rand;
extern crate tokio;
extern crate url;

use airmash_client::protocol::*;
use airmash_client::protocol::{KeyCode, ServerPacket};
use airmash_client::*;

use std::env;
use std::error::Error;
use std::time::{Duration, Instant};

use tokio::r#await;
use url::Url;

async fn on_login<'a>(client: &'a mut Client, _: &'a server::Login, _len: u64) -> ClientResult<()> {
    await!(client.wait(Duration::from_secs(3)))?;
    //await!(client.send(client::Command {
    //    com: "respawn".to_owned(),
    //    data: "2".to_owned()
    //}))?;

    Ok(())
}

async fn on_player_respawn<'a>(
    client: &'a mut Client,
    packet: &'a server::PlayerRespawn,
) -> ClientResult<()> {
    let me = client.world.me.id;

    if packet.id.0 == me {
        await!(client.press_key(KeyCode::Right))?;
        await!(client.press_key(KeyCode::Fire))?;
    }

    Ok(())
}

async fn on_packet<'a>(client: &'a mut Client, packet: ServerPacket, len: u64) -> ClientResult<()> {
    use self::ServerPacket::*;

    match packet {
        Login(x) => await!(on_login(client, &x, len))?,
        PlayerRespawn(x) => await!(on_player_respawn(client, &x))?,
        _ => (),
    }

    Ok(())
}

async fn single_bot_inner(name: String, server: Url, i: u64) -> Result<(), Box<Error + 'static>> {
    //use self::ClientEvent::*;

    let mut client = r#await!(Client::new_insecure(server))?;

    r#await!(client.wait(Duration::from_millis(100 * i)))?;

    r#await!(client.send(client::Login {
        flag: "ca".to_owned(),
        horizon_x: 3000,
        horizon_y: 3000,
        name: name,
        protocol: 5,
        session: "none".to_owned()
    }))?;

    // Should probably have a wait-for-login command
    r#await!(client.wait(Duration::from_secs(5)))?;
    //r#await!(client.send(client::Command{
    //    com: "respawn".to_owned(),
    //    data: "2".to_owned()
    //}))?;

    //let mut next = Instant::now() + Duration::from_secs(10);

    //r#await!(client.point_at(Position::new(5000.0, 5000.0)))?;

    while let Some(_) = r#await!(client.next())? {
        let player = match client.world.get_me().team.0 {
            1 => "STEAMROLLER",
            _ => "STEAMROLLER",
        };

        let id = match client.world.names.get(player) {
            Some(x) => *x,
            None => break,
        };
        r#await!(client.follow(id))?;
        r#await!(client.say("FOR GONDOR!".to_string()))?;
    }

    info!("Shutting down bot {}", client.world.get_me().name);

    Ok(())
}

async fn single_bot(name: String, server: Url, i: u64) {
    match r#await!(single_bot_inner(name, server, i)) {
        Ok(_) => (),
        Err(e) => {
            error!("The bot ended with an error {}", e);
        }
    }
}

async fn spawn_bots(name: String, url: Url) {
    for i in 0..2 {
        tokio::spawn_async(single_bot(format!("{}{}", name, i), url.clone(), i));
        r#await!(tokio::timer::Delay::new(
            Instant::now() + Duration::from_millis(100)
        ))
        .unwrap();
    }
}

fn run_bot(name: &str, server: &str) -> Result<(), Box<Error>> {
    env::set_var("RUST_BACKTRACE", "1");

    let name = name.to_owned();
    let url: Url = server.parse()?;

    tokio::run_async(spawn_bots(name, url));

    Ok(())
}

const SERVER: &'static str = "wss://game.airmash.steamroller.tk/ffa";
//const SERVER: &'static str = "wss://game-us-s1.airma.sh/ffa2";
//const SERVER: &'static str = "ws://localhost:3501";

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    if let Err(e) = run_bot("TESTBOT", SERVER) {
        println!("An error occurred:\n{}", e);
    }
}
