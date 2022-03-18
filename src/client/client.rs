use airmash_protocol::*;
use futures::prelude::*;
use futures::stream::{SplitSink, SplitStream};
use tokio::net::TcpStream;
use url::Url;

use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

use std::f32::consts::{PI, TAU};
use std::ops::{Add, Rem};
use std::time::{Duration, Instant};

use super::{ClientError, ClientResult};
use crate::consts;
use crate::game::World;

const TICKER_TIME: Duration = Duration::from_millis(16);

pub enum ClientEvent {
    Frame(Instant),
    Packet(ServerPacket),
}

pub struct Client {
    pub world: World,

    sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    next_tick: Instant,
}

// Base functions
impl Client {
    pub async fn new(url: Url) -> Result<Self, ClientError> {
        let (ws_stream, _) = connect_async(url).await?;
        let (sink, stream) = ws_stream.split();

        Ok(Self {
            world: World::default(),
            sink,
            stream,
            next_tick: Instant::now() + TICKER_TIME,
        })
    }

    async fn send_buf(&mut self, buf: Vec<u8>) -> Result<(), ClientError> {
        Ok(self.sink.send(Message::Binary(buf)).await?)
    }

    async fn packet_update<'a>(&'a mut self, packet: &'a ServerPacket) -> Result<(), ClientError> {
        use self::ServerPacket::*;
        use airmash_protocol::client::Pong;

        self.world.handle_packet(packet);

        match packet {
            Ping(p) => self.send(Pong { num: p.num }).await?,
            _ => (),
        }

        Ok(())
    }

    async fn timer_update<'a>(&'a mut self, duration: Duration) -> Result<(), ClientError> {
        self.world.update(duration.as_secs_f32() * 60.0);

        Ok(())
    }

    pub async fn send<P>(&mut self, packet: P) -> Result<(), ClientError>
    where
        P: Into<ClientPacket> + 'static,
    {
        self.send_buf(protocol::v5::serialize(&packet.into())?)
            .await?;
        Ok(())
    }

    pub async fn next(&mut self) -> Result<Option<ClientEvent>, ClientError> {
        tokio::select! {
            res = self.stream.next() => {
                let packet: ServerPacket = match res {
                    None => return Ok(None),
                    Some(Err(e)) => return Err(e.into()),
                    Some(Ok(message)) => match message {
                        Message::Binary(buf) => protocol::v5::deserialize(&buf)?,
                        Message::Ping(_) => return Ok(None),
                        Message::Pong(_) => return Ok(None),
                        Message::Text(txt) => {
                            return Err(ClientError::InvalidWsFrame(format!(
                                "Server sent a text frame with body: {:?}",
                                txt
                            )));
                        }
                        Message::Close(_) => panic!(),
                        _ => unimplemented!()
                    },
                };

                self.packet_update(&packet).await?;
                return Ok(Some(ClientEvent::Packet(packet)));
            }
            _ = tokio::time::sleep_until(self.next_tick.into()) => {
                let tick = self.next_tick;
                self.next_tick += TICKER_TIME;
                self.timer_update(TICKER_TIME).await?;
                return Ok(Some(ClientEvent::Frame(tick)));
            }
        }
    }
}

// Helper functions
impl Client {
    /// Press or release a key.
    ///
    /// This corresponds to the [`Key`] client packet.
    ///
    /// [`Key`]: protocol::client::Key
    pub async fn send_key(&mut self, key: KeyCode, state: bool) -> ClientResult<()> {
        use airmash_protocol::client::Key;

        let seq = self.world.key_seq;
        self.world.key_seq += 1;

        self.send(Key { key, seq, state }).await
    }

    /// Press a key.
    ///
    /// This corresponds to calling [`send_key`] with `true`.
    pub async fn press_key(&mut self, key: KeyCode) -> ClientResult<()> {
        self.send_key(key, true).await
    }

    /// Release a key.
    ///
    /// This corresponds to calling [`send_key`] with false.
    pub async fn release_key(&mut self, key: KeyCode) -> ClientResult<()> {
        self.send_key(key, false).await
    }

    /// Process events until the target time passes.
    pub async fn wait_until(&mut self, tgt: Instant) -> ClientResult<()> {
        while let Some(evt) = self.next().await? {
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
        self.wait_until(Instant::now() + dur).await
    }

    /// Turn the plane by a given rotation.
    ///
    /// This is a best effort implementation as it is
    /// impossible to turn exactly any given amount.
    /// This method may overshoot in cases where network
    /// ping changes significantly during the execution
    /// of the turn.
    pub async fn turn(&mut self, rot: Rotation) -> ClientResult<()> {
        let rotrate = consts::rotation_rate(self.world.get_me().plane);
        let time = Duration::from_secs_f32((rot.abs() / rotrate).min(100.0) / 60.0);

        let key = if rot < 0.0.into() {
            KeyCode::Left
        } else {
            KeyCode::Right
        };

        if rot.abs() < 0.05 {
            return Ok(());
        }

        self.press_key(key).await?;
        self.wait(time).await?;
        self.release_key(key).await?;

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
        fn modulus<T>(a: T, b: T) -> T
        where
            T: Rem<Output = T> + Add<Output = T> + Copy,
        {
            (a % b + b) % b
        }

        // Determine the shortest turn angle
        // The basic idea comes from this SO answer
        // https://stackoverflow.com/questions/9505862/shortest-distance-between-two-degree-marks-on-a-circle
        let rot = self.world.get_me().rot;
        let mut dist = modulus(tgt - rot, TAU);

        if dist > PI {
            dist -= TAU;
        }

        self.turn(dist).await
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

        let rel = (pos - self.world.get_me().pos).normalize();
        let mut angle = Vector2::dot(&rel, &BASE_DIR).acos();

        if rel.x < 0.0.into() {
            angle = 2.0 * PI - angle;
        }

        self.turn_to(angle.into()).await
    }

    /// Say something in chat
    pub async fn chat(&mut self, text: String) -> ClientResult<()> {
        self.send(client::Chat { text: text.into() }).await
    }

    /// Say something in a text bubble
    pub async fn team_chat(&mut self, text: String) -> ClientResult<()> {
        self.send(client::TeamChat { text: text.into() }).await
    }

    /// Say something in a text bubble
    pub async fn say(&mut self, text: String) -> ClientResult<()> {
        self.send(client::Say { text: text.into() }).await
    }

    pub async fn wait_for_login(&mut self) -> ClientResult<Option<server::Login>> {
        use self::ClientEvent::*;

        while let Some(x) = self.next().await? {
            if let Packet(ServerPacket::Login(p)) = x {
                return Ok(Some(p));
            }
        }

        Ok(None)
    }
}
