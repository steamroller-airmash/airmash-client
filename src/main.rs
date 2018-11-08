extern crate airmash_client;
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate tokio;

use airmash_client::protocol::{KeyCode, PlaneType, Protocol};
use airmash_client::protocol::server::{PlayerRespawn};
use airmash_client::{ClientStream, Client, ClientState, ClientResult};
use futures::Future;

use std::env;
use std::error::Error;
use std::time::Duration;
use tokio::prelude::future as futures;

struct MoveForwardAndShoot;

impl<P: Protocol> Client<P> for MoveForwardAndShoot {
    fn on_player_respawn<'a>(&mut self, state: &ClientState<'a, P>, packet: &PlayerRespawn) -> ClientResult<P> {
        let me = state.state().me.id;

        if packet.id == me {
            state.press_key(KeyCode::Up)?;
            state.press_key(KeyCode::Fire)?;
        }

        Ok(())
    }

    fn on_close<'a>(&mut self, _: &ClientState<'a, P>) -> ClientResult<P> {
        println!("Closing!");
        Ok(())
    }
}

fn run_bot(name: &str, server: &str) -> Result<(), Box<Error>> {
    env::set_var("RUST_BACKTRACE", "1");

    let mut vals = vec![];

    for i in 0..100 {
        let wait = i % 100;
        let client = ClientStream::new(server)?
            .login(&format!("{} {}", name, i), "CA")
            .wait(Duration::from_secs(2))
            .wait(Duration::from_millis(wait * 20))
            .wait(Duration::from_secs(5))
            .switch_plane(PlaneType::Goliath)
            .press_key(KeyCode::Fire)
            .press_key(KeyCode::Up)
            .into_boxed()
            //.enter_spectate()
            .with_client(MoveForwardAndShoot)
            .until_close()
            .map_err(|e| { error!("An error occurred: {}", e); })
            .map(move |_| error!("Closed client {}", i));
        vals.push(client);
    }

    tokio::run(futures::join_all(vals).map(|_| ()));

    Ok(())
}

//const SERVER: &'static str = "wss://game.airmash.steamroller.tk/dev";
const SERVER: &'static str = /*"wss://game-eu-s1.airma.sh/ctf1";// */"ws://localhost:3501";

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    if let Err(e) = run_bot("DEATHBOT", SERVER) {
        println!("An error occurred:\n{}", e);
    }
}
