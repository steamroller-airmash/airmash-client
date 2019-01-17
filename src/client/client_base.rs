use url::Url;

use std::net::ToSocketAddrs;
use std::ops::{Deref, DerefMut};
use std::time::{Duration, Instant};

use tokio::prelude::*;
use tokio::r#await;
use tokio::timer::Interval;
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use airmash_protocol::*;
use airmash_protocol_v5::ProtocolV5;

use crate::future::BoxedFuture;
use crate::game::World;
use crate::ClientEvent;
use crate::{Client, ClientError, ClientFuture, ClientResult, ImplClient};

static TICKER_TIME: Duration = Duration::from_millis(16);

type FromFn<T, U> = fn(T) -> U;
type ParseTimeFn = fn(std::time::Instant) -> ClientEvent;
type ParsePacketFn = fn(tungstenite::Message) -> Result<Option<ClientEvent>, ClientError>;

type WebSocketStream = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::stream::Stream<
        tokio::net::TcpStream,
        tokio_tls::TlsStream<tokio::net::TcpStream>,
    >,
>;

type ClientSink = futures::stream::SplitSink<WebSocketStream>;

// This is ugly, but it means that client doesn't need type parameters
type ClientStream = futures::stream::Fuse<
    futures::stream::Select<
        futures::stream::FilterMap<
            futures::stream::AndThen<
                futures::stream::MapErr<
                    futures::stream::SplitStream<WebSocketStream>,
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

pub struct ClientBase {
    world: World,
    sink: Option<ClientSink>,
    stream: ClientStream,
}

impl Client for ClientBase {
    fn world(&mut self) -> &mut World {
        &mut self.world
    }

    fn _next<'a>(&'a mut self) -> ClientFuture<'a, Option<ClientEvent>> {
        Box::new(self._next_impl())
    }

    fn _send_buf<'a>(&'a mut self, buf: Vec<u8>) -> ClientFuture<'a, ()> {
        Box::new(self._send_buf_impl(buf))
    }
}

/// async trait backing functions
impl ClientBase {
    async fn _send_buf_impl(&mut self, buf: Vec<u8>) -> Result<(), ClientError> {
        let sink = self.sink.take().unwrap();
        let msg = Message::Binary(buf);

        self.sink = Some(r#await!(sink.send(msg))?);

        Ok(())
    }

    async fn _next_impl(&mut self) -> Result<Option<ClientEvent>, ClientError> {
        use self::ClientEvent::*;

        let val = match r#await!(self.stream.next()) {
            Some(x) => x?,
            None => return Ok(None),
        };

        match &val {
            Packet(p) => r#await!(self.packet_update(p))?,
            Frame(now) => self.world.update(*now),
        }

        Ok(Some(val))
    }

    async fn send_buf(&mut self, buf: Vec<u8>) -> ClientResult<()> {
        r#await!(BoxedFuture::new(self._send_buf(buf)))
    }

    async fn send<P>(&mut self, packet: P) -> Result<(), ClientError>
    where
        P: Into<ClientPacket> + 'static,
    {
        let packets: Vec<_> = ProtocolV5 {}.serialize_client(&packet.into())?.collect();

        for buf in packets {
            r#await!(self.send_buf(buf))?;
        }

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
}

/// Constructors
impl ClientBase {
    fn new_internal(ws_stream: WebSocketStream) -> Self {
        let (sink, stream) = ws_stream.split();

        let stream1 = stream
            .map_err(ClientError::from as FromFn<_, _>)
            .and_then(parse_packet as ParsePacketFn)
            .filter_map(id as fn(_) -> _);
        let stream2 = Interval::new(Instant::now(), TICKER_TIME)
            .map_err(ClientError::from as FromFn<_, _>)
            .map(parse_time as ParseTimeFn);

        Self {
            world: World::default(),
            sink: Some(sink),
            stream: stream1.select(stream2).fuse(),
        }
    }
    async fn from_tls_stream(
        url: Url,
        stream: tokio_tls::TlsStream<tokio::net::TcpStream>,
    ) -> Result<Self, ClientError> {
        use tokio_tungstenite::client_async;
        use tokio_tungstenite::stream::Stream;

        let (ws_stream, _) = r#await!(client_async(url, Stream::Tls(stream)))?;

        Ok(Self::new_internal(ws_stream))
    }

    pub async fn new(url: Url) -> Result<Self, ClientError> {
        let (ws_stream, _) = r#await!(connect_async(url))?;

        Ok(Self::new_internal(ws_stream))
    }
    pub async fn new_insecure(url: Url) -> Result<Self, ClientError> {
        let stream = r#await!(connect_insecure(&url))?;

        r#await!(Self::from_tls_stream(url, stream))
    }
}

impl Deref for ClientBase {
    type Target = ImplClient<Self>;

    fn deref(&self) -> &ImplClient<Self> {
        unsafe { std::mem::transmute(self) }
    }
}

impl DerefMut for ClientBase {
    fn deref_mut(&mut self) -> &mut ImplClient<Self> {
        unsafe { std::mem::transmute(self) }
    }
}

fn id<T>(v: T) -> T {
    v
}

fn parse_packet(msg: Message) -> Result<Option<ClientEvent>, ClientError> {
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

async fn connect_insecure(
    server: &Url,
) -> Result<tokio_tls::TlsStream<tokio::net::TcpStream>, ClientError> {
    use tokio::net::TcpStream;
    use tokio_tls::TlsConnector;

    let socket_addr = server
        .to_socket_addrs()
        .map_err(|x| ClientError::WebSocket(x.into()))?
        .next()
        // FIXME: Create a result instead
        .expect("Provided URL did not map to an address");

    let tcp_stream =
        r#await!(TcpStream::connect(&socket_addr)).map_err(|x| ClientError::WebSocket(x.into()))?;
    let nattls_connector = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|x| ClientError::WebSocket(x.into()))?;

    let stream = TlsConnector::from(nattls_connector).connect(server.as_str(), tcp_stream);

    let stream = r#await!(stream).map_err(|x| ClientError::WebSocket(x.into()))?;

    Ok(stream)
}

fn parse_time(inst: Instant) -> ClientEvent {
    ClientEvent::Frame(inst)
}
