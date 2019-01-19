use url::Url;

use std::f32::consts::PI;
use std::future::Future;
use std::ops::{Add, Rem};
use std::r#await as std_await;
use std::time::{Duration, Instant};

use tokio::r#await;

use airmash_protocol::*;
use airmash_protocol_v5::ProtocolV5;

use super::{ClientBase, ClientError, ClientResult};
use crate::consts;
use crate::future::BoxedFuture;
use crate::game::World;
use crate::ClientEvent;

pub type ClientFuture<'a, T> = Box<dyn Future<Output = Result<T, ClientError>> + Send + 'a>;

pub trait Client {
    fn world(&self) -> &World;
    fn world_mut(&mut self) -> &mut World;
    fn _next<'a>(&'a mut self) -> ClientFuture<'a, Option<ClientEvent>>;
    fn _send_buf<'a>(&'a mut self, buf: Vec<u8>) -> ClientFuture<'a, ()>;
}

// Hack as described here https://github.com/rust-lang/rfcs/issues/1971#issuecomment-294282433
pub struct ImplClient<T: Client + Sized>(T);

// Bridge functions
impl<T: Client> ImplClient<T> {
    pub(crate) fn world(&self) -> &World {
        self.0.world()
    }
    pub(crate) fn world_mut(&mut self) -> &mut World {
        self.0.world_mut()
    }
    pub(self) fn _next<'a>(&'a mut self) -> ClientFuture<'a, Option<ClientEvent>> {
        self.0._next()
    }
    pub(self) fn _send_buf<'a>(&'a mut self, buf: Vec<u8>) -> ClientFuture<'a, ()> {
        self.0._send_buf(buf)
    }
}

impl Client {
    pub async fn new(url: Url) -> Result<ClientBase, ClientError> {
        r#await!(ClientBase::new(url))
    }
    pub async fn new_insecure(url: Url) -> Result<ClientBase, ClientError> {
        r#await!(ClientBase::new_insecure(url))
    }
}

// Base functions
impl<T: Client> ImplClient<T> {
    pub async fn next(&mut self) -> ClientResult<Option<ClientEvent>> {
        r#await!(BoxedFuture::new(self._next()))
    }

    pub async fn send_buf(&mut self, buf: Vec<u8>) -> ClientResult<()> {
        r#await!(BoxedFuture::new(self._send_buf(buf)))
    }

    pub async fn send<P>(&mut self, packet: P) -> Result<(), ClientError>
    where
        P: Into<ClientPacket> + 'static,
    {
        let packets: Vec<_> = ProtocolV5 {}.serialize_client(&packet.into())?.collect();

        for buf in packets {
            r#await!(self.send_buf(buf))?;
        }

        Ok(())
    }
}

// Helper functions
impl<T: Client> ImplClient<T> {
    /// Press or release a key.
    ///
    /// This corresponds to the [`Key`] client packet.
    ///
    /// [`Key`]: protocol::client::Key
    pub async fn send_key(&mut self, key: KeyCode, state: bool) -> ClientResult<()> {
        use airmash_protocol::client::Key;

        let seq = self.world().key_seq;
        self.world_mut().key_seq += 1;

        r#await!(self.send(Key { key, seq, state }))
    }

    /// Press a key.
    ///
    /// This corresponds to calling [`send_key`] with `true`.
    pub async fn press_key(&mut self, key: KeyCode) -> ClientResult<()> {
        r#await!(self.send_key(key, true))
    }

    /// Release a key.
    ///
    /// This corresponds to calling [`send_key`] with false.
    pub async fn release_key(&mut self, key: KeyCode) -> ClientResult<()> {
        r#await!(self.send_key(key, false))
    }

    /// Process events until the target time passes.
    pub async fn wait_until(&mut self, tgt: Instant) -> ClientResult<()> {
        while let Some(evt) = r#await!(self.next())? {
            if let ClientEvent::Frame(frame) = evt {
                if frame > tgt {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Process events for the given duration.
    pub async fn wait(&mut self, dur: Duration) -> ClientResult<()> {
        r#await!(self.wait_until(Instant::now() + dur))
    }

    /// Turn the plane by a given rotation.
    ///
    /// This is a best effort implementation as it is
    /// impossible to turn exactly any given amount.
    /// This method may overshoot in cases where network
    /// ping changes significantly during the execution
    /// of the turn.
    pub async fn turn(&mut self, rot: Rotation) -> ClientResult<()> {
        let rotrate = consts::rotation_rate(self.world().get_me().plane);
        let time: Duration = (rot.abs() / rotrate).min(Time::new(100.0)).into();

        let key = if rot < 0.0.into() {
            KeyCode::Left
        } else {
            KeyCode::Right
        };

        if rot.inner().abs() < 0.05 {
            return Ok(());
        }

        r#await!(self.press_key(key))?;
        r#await!(self.wait(time))?;
        r#await!(self.release_key(key))?;

        Ok(())
    }

    /// Turn to a given angle.
    ///
    /// This is a best effort implementation as it is
    /// impossible to turn exactly any given amount.
    /// This method may overshoot in cases where network
    /// ping changes significantly during the execution
    /// of the turn.
    pub async fn turn_to(&mut self, tgt: Rotation) -> ClientResult<()> {
        /// Utility since rust doesn't provide fmod
        fn fmod<T>(a: T, b: T) -> T
        where
            T: Rem<Output = T> + Add<Output = T> + Copy,
        {
            (a % b + b) % b
        }

        // Determine the shortest turn angle
        // The basic idea comes from this SO answer
        // https://stackoverflow.com/questions/9505862/shortest-distance-between-two-degree-marks-on-a-circle
        let rot = self.world().get_me().rot;
        let pi = Rotation::new(PI);
        let pi2 = 2.0 * pi;
        let mut dist = fmod(tgt - rot, pi2);

        if dist > pi {
            dist -= pi2;
        }

        r#await!(self.turn(dist))
    }

    /// Point the plane at a given point.
    ///
    /// This is a best effort implementation as it is
    /// impossible to turn exactly any given amount.
    /// This method may overshoot in cases where network
    /// ping changes significantly during the execution
    /// of the turn.
    pub async fn point_at(&mut self, pos: Position) -> ClientResult<()> {
        use crate::consts::BASE_DIR;

        let rel = (pos - self.world().get_me().pos).normalized();
        let mut angle = Vector2::dot(rel, BASE_DIR).acos();

        if rel.x < 0.0.into() {
            angle = 2.0 * PI - angle;
        }

        r#await!(self.turn_to(angle.into()))
    }

    /// Say something in chat
    pub async fn chat(&mut self, text: String) -> ClientResult<()> {
        r#await!(self.send(client::Chat { text }))
    }

    /// Say something in a text bubble
    pub async fn team_chat(&mut self, text: String) -> ClientResult<()> {
        r#await!(self.send(client::TeamChat { text }))
    }

    /// Say something in a text bubble
    pub async fn say(&mut self, text: String) -> ClientResult<()> {
        r#await!(self.send(client::Say { text }))
    }

    /// Wait to receive a login packet. If the
    /// connection closes before receiving the
    /// packet then it will return `None`.
    pub async fn wait_for_login(&mut self) -> ClientResult<Option<server::Login>> {
        use self::ClientEvent::*;

        while let Some(x) = r#await!(self.next())? {
            if let Packet(ServerPacket::Login(p)) = x {
                return Ok(Some(p));
            }
        }

        Ok(None)
    }
}
