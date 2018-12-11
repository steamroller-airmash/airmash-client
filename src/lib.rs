//!

#![feature(futures_api)]

extern crate ws;
#[macro_use]
extern crate log;
#[macro_use]
extern crate derivative;
pub extern crate airmash_protocol as protocol;
extern crate airmash_protocol_v5 as protocol_v5;
extern crate fnv;
extern crate futures;
extern crate hashbrown;
extern crate tokio;
extern crate tokio_tungstenite;
extern crate url;

use tokio_tungstenite::tungstenite;

mod client;
mod client_trait;
mod error;
mod game;
mod gamestate;
mod message_handler;
mod received_message;

mod client2;

pub use crate::client::{ClientBase, ClientEvent, ClientEventData, ClientStream};
pub use crate::client_trait::{default_on_packet, Client, ClientResult, ClientState};
pub use crate::error::ClientError;
pub use crate::gamestate::{GameState, MobData, MyPlayerData, PlayerData};
