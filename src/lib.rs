//!

#![feature(core_intrinsics)]

#[macro_use]
extern crate log;
pub extern crate airmash_protocol as protocol;
extern crate futures;
extern crate hashbrown;
extern crate tokio;
extern crate tokio_tungstenite;
extern crate tungstenite;
extern crate url;

mod client;
mod game;
mod config;
//mod map;

pub mod consts;

pub use self::client::*;
pub use self::game::*;
pub use self::config::Config;
