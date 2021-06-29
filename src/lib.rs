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
//mod map;

pub mod consts;

pub use self::client::*;
pub use self::game::*;
