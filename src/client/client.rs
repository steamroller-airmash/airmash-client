use url::Url;

use std::time::{Duration, Instant};

use tokio::prelude::*;
use tokio::r#await;
use tokio::timer::Interval;
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use futures::{Sink, Stream};

use airmash_protocol::{ClientPacket, KeyCode, Protocol, ServerPacket};
use airmash_protocol_v5::ProtocolV5;

use super::*;
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
type ParsePacketFn = fn(tungstenite::Message) -> Result<ClientEvent, ClientError>;

type ClientStream = futures::stream::Fuse<
    futures::stream::Select<
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
            Result<ClientEvent, ClientError>,
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

fn parse_packet(msg: Message) -> Result<ClientEvent, ClientError> {
    let buf = match msg {
        Message::Binary(buf) => buf,
        _ => return Err(ClientError::InvalidWsFrame),
    };

    ProtocolV5 {}
        .deserialize_server(&buf)
        .map(ClientEvent::Packet)
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
            .and_then(parse_packet as ParsePacketFn);
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
        let val = match r#await!(self.stream.next()) {
            Some(x) => x?,
            None => return Ok(None),
        };

        Ok(Some(val))
    }
}

// Helper functions
impl Client {
    pub async fn send_key(&mut self, key: KeyCode, state: bool) -> ClientResult<()> {
        use airmash_protocol::client::Key;

        let seq = self.world.key_seq;
        self.world.key_seq += 1;

        r#await!(self.send(Key { key, seq, state }))
    }

    pub async fn press_key(&mut self, key: KeyCode) -> ClientResult<()> {
        r#await!(self.send_key(key, true))
    }

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

    pub async fn wait(&mut self, dur: Duration) -> ClientResult<()> {
        r#await!(self.wait_until(Instant::now() + dur))
    }
}
