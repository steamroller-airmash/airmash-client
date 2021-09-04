//!

#![feature(core_intrinsics)]

#[macro_use]
extern crate log;

pub extern crate airmash_protocol as protocol;

mod client;
mod game;
mod config;
mod util;
// pub mod map;

pub mod consts;

pub use self::client::*;
pub use self::game::*;
pub use self::config::Config;
