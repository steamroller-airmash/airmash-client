use url::Url;

use std::f32::consts::PI;
use std::ops::{Add, Rem};
use std::time::{Duration, Instant};

use tokio::prelude::*;
use tokio::r#await;
use tokio::timer::Interval;
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use futures::{Sink, Stream};

use airmash_protocol::*;
use airmash_protocol_v5::ProtocolV5;

use super::{ClientError, ClientResult};
use crate::consts;
use crate::game::World;

type ClientSink = futures::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::stream::Stream<
            tokio::net::TcpStream,
            tokio_tls::TlsStream<tokio::net::TcpStream>,
        >,
    >,
>;

type FromFn<T, U> = fn(T) -> U;
type ParseTimeFn = fn(std::time::Instant) -> ClientEvent;
type ParsePacketFn = fn(tungstenite::Message) -> Result<Option<ClientEvent>, ClientError>;

// This is ugly, but it means that client doesn't need type parameters
type ClientStream = futures::stream::Fuse<
    futures::stream::Select<
        futures::stream::FilterMap<
            futures::stream::AndThen<
                futures::stream::MapErr<
                    futures::stream::SplitStream<
                        tokio_tungstenite::WebSocketStream<
                            tokio_tungstenite::stream::Stream<
                                tokio::net::TcpStream,
                                tokio_tls::TlsStream<tokio::net::TcpStream>,
                            >,
                        >,
                    >,
                    FromFn<tungstenite::Error, ClientError>,
                >,
                ParsePacketFn,
                Result<Option<ClientEvent>, ClientError>,
            >,
            fn(Option<ClientEvent>) -> Option<ClientEvent>,
        >,
        futures::stream::Map<
            futures::stream::MapErr<Interval, FromFn<tokio::timer::Error, ClientError>>,
            ParseTimeFn,
        >,
    >,
>;

static TICKER_TIME: Duration = Duration::from_millis(16);

pub enum ClientEvent {
    Frame(Instant),
    Packet(ServerPacket),
}

pub struct Client {
    pub world: World,
    sink: Option<ClientSink>,
    stream: ClientStream,
}

fn id<T>(v: T) -> T {
    v
}

fn parse_packet(msg: Message) -> Result<Option<ClientEvent>, ClientError> {
    //unsafe { std::intrinsics::breakpoint() };

    let buf = match msg {
        Message::Binary(buf) => buf,
        Message::Ping(_) => return Ok(None),
        Message::Pong(_) => return Ok(None),
        Message::Text(txt) => {
            return Err(ClientError::InvalidWsFrame(format!(
                "Server sent a text frame with body: {:?}",
                txt
            )));
        }
    };

    ProtocolV5 {}
        .deserialize_server(&buf)
        .map(ClientEvent::Packet)
        .map(Some)
        .map_err(Into::into)
}

fn parse_time(inst: Instant) -> ClientEvent {
    ClientEvent::Frame(inst)
}

// Base functions
impl Client {
    pub async fn new(url: Url) -> Result<Self, ClientError> {
        let (ws_stream, _) = r#await!(connect_async(url))?;

        let (sink, stream) = ws_stream.split();

        let stream1 = stream
            .map_err(ClientError::from as FromFn<_, _>)
            .and_then(parse_packet as ParsePacketFn)
            .filter_map(id as fn(_) -> _);
        let stream2 = Interval::new(Instant::now(), TICKER_TIME)
            .map_err(ClientError::from as FromFn<_, _>)
            .map(parse_time as ParseTimeFn);

        Ok(Self {
            world: World::default(),
            sink: Some(sink),
            stream: stream1.select(stream2).fuse(),
        })
    }

    async fn send_buf(&mut self, buf: Vec<u8>) -> Result<(), ClientError> {
        let sink = self.sink.take().unwrap();
        let msg = Message::Binary(buf);

        self.sink = Some(r#await!(sink.send(msg))?);

        Ok(())
    }

    async fn packet_update<'a>(&'a mut self, packet: &'a ServerPacket) -> Result<(), ClientError> {
        use self::ServerPacket::*;
        use airmash_protocol::client::Pong;

        self.world.handle_packet(packet);

        match packet {
            Ping(p) => r#await!(self.send(Pong { num: p.num }))?,
            _ => (),
        }

        Ok(())
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

    pub async fn next(&mut self) -> Result<Option<ClientEvent>, ClientError> {
        use self::ClientEvent::*;

        let val = match r#await!(self.stream.next()) {
            Some(x) => x?,
            None => return Ok(None),
        };

        match &val {
            Packet(p) => r#await!(self.packet_update(p))?,
            Frame(_) => (),
        }

        Ok(Some(val))
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
        let rotrate = consts::rotation_rate(self.world.get_me().plane);
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
        let pi = Rotation::new(PI);
        let pi2 = 2.0 * pi;
        let mut dist = modulus(tgt - rot, pi2);

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

        let rel = (pos - self.world.get_me().pos).normalized();
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
