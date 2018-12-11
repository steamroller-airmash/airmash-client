//!

#![feature(futures_api, await_macro)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate derivative;
pub extern crate airmash_protocol as protocol;
extern crate airmash_protocol_v5 as protocol_v5;
extern crate futures;
extern crate hashbrown;
extern crate tokio;
extern crate tokio_tls;
extern crate tokio_tungstenite;
extern crate tungstenite;
extern crate url;

mod game;
mod client;

use self::game::*;
use self::client::*;
