//!

#![feature(futures_api, await_macro, async_await)]

#[macro_use]
extern crate log;
pub extern crate airmash_protocol as protocol;
extern crate airmash_protocol_v5 as protocol_v5;
extern crate futures;
extern crate hashbrown;
extern crate tokio;
extern crate tokio_tls;
extern crate tokio_tungstenite;
extern crate tungstenite;
extern crate url;

mod client;
mod config;
mod future;
mod game;
mod macros;
//mod map;

pub mod consts;

pub use self::client::*;
pub use self::game::*;
