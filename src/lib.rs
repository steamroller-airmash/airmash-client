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

mod client;
mod client_trait;
mod error;
mod gamestate;
mod message_handler;
mod public_message;
mod received_message;

pub use client::{ClientStream, ClientBase, ClientEvent, ClientEventData};
pub use client_trait::{Client, ClientState, default_on_packet};
pub use error::ClientError;
