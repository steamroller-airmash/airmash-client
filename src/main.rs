#![feature(futures_api, await_macro, async_await)]

extern crate airmash_client;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate tokio;
extern crate url;

use airmash_client::protocol::{client, server};
use airmash_client::protocol::{KeyCode, ServerPacket};
use airmash_client::*;

use std::env;
use std::error::Error;
use std::time::Duration;

use tokio::r#await;
use url::Url;

async fn on_login<'a>(client: &'a mut Client, _: &'a server::Login, _len: u64) -> ClientResult<()> {
    await!(client.wait(Duration::from_secs(3)))?;
    await!(client.send(client::Command {
        com: "respawn".to_owned(),
        data: "3".to_owned()
    }))?;

    Ok(())
}

async fn on_player_respawn<'a>(
    client: &'a mut Client,
    packet: &'a server::PlayerRespawn,
) -> ClientResult<()> {
    let me = client.world.me.id;

    if packet.id.0 == me {
        await!(client.press_key(KeyCode::Right))?;
        await!(client.press_key(KeyCode::Up))?;
        await!(client.press_key(KeyCode::Fire))?;
    }

    Ok(())
}

async fn on_packet<'a>(client: &'a mut Client, packet: ServerPacket, len: u64) -> ClientResult<()> {
    use self::ServerPacket::*;

    client.world.handle_packet(&packet);

    match packet {
        Login(x) => await!(on_login(client, &x, len))?,
        PlayerRespawn(x) => await!(on_player_respawn(client, &x))?,
        Ping(x) => await!(client.send(client::Pong { num: x.num }))?,
        _ => (),
    }

    Ok(())
}

async fn single_bot_inner(name: String, server: Url, i: u64) -> Result<(), Box<Error + 'static>> {
    use self::ClientEvent::*;

    let mut client = r#await!(Client::new(server))?;

    r#await!(client.wait(Duration::from_millis(100 * i)))?;

    r#await!(client.send(client::Login {
        flag: "jolly".to_owned(),
        horizon_x: 4500,
        horizon_y: 4500,
        name: name,
        protocol: 5,
        session: "none".to_owned()
    }))?;

    while let Some(evt) = r#await!(client.next())? {
        match evt {
            Packet(p) => r#await!(on_packet(&mut client, p, i))?,
            Frame(_) => (),
        }
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
    for i in 0..30 {
        tokio::spawn_async(single_bot(format!("{}{}", name, i), url.clone(), i));
    }
}

fn run_bot(name: &str, server: &str) -> Result<(), Box<Error>> {
    env::set_var("RUST_BACKTRACE", "1");

    let name = name.to_owned();
    let url: Url = server.parse()?;

    tokio::run_async(spawn_bots(name, url));

    Ok(())
}

const SERVER: &'static str = "wss://game.airmash.steamroller.tk/dev";
//const SERVER: &'static str = "wss://game-asia-s1.airma.sh/ctf1";
//const SERVER: &'static str = "ws://localhost:3501";

fn main() {
    env_logger::init();

    if let Err(e) = run_bot("DEATHBOT", SERVER) {
        println!("An error occurred:\n{}", e);
    }
}
