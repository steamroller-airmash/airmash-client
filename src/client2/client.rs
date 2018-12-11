use url::Url;

use tokio::prelude::*;
use tokio::r#await;
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use futures::{Sink, Stream};

use airmash_protocol::{ClientPacket, Protocol, ServerPacket};
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
type ClientStream = futures::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::stream::Stream<
            tokio::net::TcpStream,
            tokio_tls::TlsStream<tokio::net::TcpStream>,
        >,
    >,
>;

pub struct Client {
    pub world: World,
    sink: Option<ClientSink>,
    stream: ClientStream,
}

impl Client {
    pub async fn new(url: Url) -> Result<Self, ClientError> {
        let (ws_stream, _) = r#await!(connect_async(url))?;

        let (sink, stream) = ws_stream.split();

        Ok(Self {
            world: World::default(),
            sink: Some(sink),
            stream: stream,
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
        let packets = ProtocolV5 {}.serialize_client(&packet.into())?;

        for buf in packets {
            r#await!(self.send_buf(buf))?;
        }

        Ok(())
    }

    pub async fn next(&mut self) -> Option<Result<ServerPacket, ClientError>> {
        let msg = match r#await!(self.stream.next())? {
            Ok(x) => x,
            Err(e) => return Some(Err(e.into())),
        };

        let buf = match msg {
            Message::Binary(buf) => buf,
            _ => return Some(Err(ClientError::InvalidWsFrame)),
        };

        Some(ProtocolV5 {}.deserialize_server(&buf).map_err(Into::into))
    }
}
