//!
//!

extern crate ws;
#[macro_use]
extern crate log;
#[macro_use]
extern crate derivative;
pub extern crate airmash_protocol as protocol;
extern crate airmash_protocol_v5 as protocol_v5;
extern crate fnv;
extern crate futures;
extern crate tokio;
extern crate hashbrown;

mod client;
mod client_trait;
mod error;
mod game;
mod gamestate;
mod message_handler;
mod received_message;

pub use client::{ClientBase, ClientEvent, ClientEventData, ClientStream};
pub use client_trait::{default_on_packet, Client, ClientResult, ClientState};
pub use error::ClientError;
pub use gamestate::{GameState, MobData, MyPlayerData, PlayerData};
