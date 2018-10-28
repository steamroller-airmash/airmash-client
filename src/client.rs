use futures::sync::mpsc::{unbounded, UnboundedReceiver as Receiver};
use futures::{Future, Stream};
use tokio::timer::Interval;

use protocol::{ClientPacket, KeyCode, Protocol, ServerPacket, PlaneType};
use protocol_v5::ProtocolV5;

use ws::Sender;

use std::borrow::Borrow;
use std::mem;
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, Instant};

use error::{ClientError, PacketSerializeError};
use gamestate::GameState;
use message_handler::websocket_runner;
use received_message::{ReceivedMessage, ReceivedMessageData};

const FRAME_TIME: Duration = Duration::from_nanos(16666667);

enum InternalEvent {
    Frame(Instant),
    Packet(ReceivedMessage),
}

#[derive(Clone, Debug)]
pub enum ClientEventData {
    Frame,
    Packet(ServerPacket),
    Close,
}

#[derive(Clone)]
pub struct ClientEvent<P: Protocol> {
    pub inst: Instant,
    pub data: ClientEventData,
    pub base: Arc<ClientBase<P>>,
    pub state: Arc<Mutex<GameState>>,
    pub key_seq: Arc<Mutex<u32>>,
}

pub struct ClientBase<P: Protocol> {
    #[allow(dead_code)]
    message_thread: JoinHandle<()>,
    sender: Sender,
    pub protocol: P,
}

fn build_select_stream<P: Protocol>(
    channel: Receiver<ReceivedMessage>,
) -> impl Stream<Item = InternalEvent, Error = ClientError<P>> {
    let timer_stream = Interval::new(Instant::now(), FRAME_TIME)
        .map(InternalEvent::Frame)
        .map_err(|e| -> ClientError<P> { e.into() });
    let message_stream = channel
        .map(InternalEvent::Packet)
        .map_err(|_| -> ClientError<P> { unimplemented!() });

    timer_stream.select(message_stream)
}

fn build_client_stream(
    addr: String,
) -> Result<
    impl Stream<Item = ClientEvent<ProtocolV5>, Error = ClientError<ProtocolV5>>,
    ClientError<ProtocolV5>,
> {
    use self::ClientEventData::*;

    let (client, channel) = ClientBase::<ProtocolV5>::new(addr)?;
    let gamestate = Arc::new(Mutex::new(GameState::default()));
    let client = Arc::new(client);
    let key_seq = Arc::new(Mutex::new(0));

    let select_stream = build_select_stream(channel);

    Ok(select_stream
        .filter_map(move |val| {
            let (data, inst) = {
                let lock = gamestate.lock();
                let ref mut gamestate = lock.unwrap();

                match &val {
                    InternalEvent::Packet(p) => {
                        if p.is_close() {
                            (Close, p.time)
                        } else {
                            match p.as_packet(&client.protocol) {
                                Ok(sp) => {
                                    gamestate.update_state(&sp);
                                    (Packet(sp), p.time)
                                }
                                Err(_) => return None,
                            }
                        }
                    }
                    InternalEvent::Frame(i) => {
                        gamestate.update_frame(*i);
                        (Frame, *i)
                    }
                }
            };

            return Some(ClientEvent {
                inst: inst,
                data: data,
                base: Arc::clone(&client),
                state: Arc::clone(&gamestate),
                key_seq: Arc::clone(&key_seq),
            });
        })
        .take_while(move |evt| {
            Ok(match evt.data {
                Close => false,
                _ => true,
            })
        })
        .and_then(|evt| {
            use self::ClientEventData::*;
            use protocol::client::Pong;
            use protocol::ServerPacket::Ping;

            if let Packet(ref packet) = evt.data {
                if let Ping(p) = packet {
                    evt.base.send_packet(Pong { num: p.num })?;
                }
            }

            Ok(evt)
        }))
}

impl<P: Protocol + 'static> ClientBase<P> {
    fn create_event_thread(addr: String) -> (JoinHandle<()>, Receiver<ReceivedMessage>) {
        let (message_send, message_recv) = unbounded();
        let handle = spawn(move || websocket_runner(addr, message_send));

        (handle, message_recv)
    }

    fn build_with_protocol(
        addr: String,
        protocol: P,
    ) -> Result<(Self, Receiver<ReceivedMessage>), ClientError<P>> {
        let (handle, channel) = Self::create_event_thread(addr);

        let mut wait = channel.wait();

        let sender = match wait.next().unwrap().unwrap().data {
            ReceivedMessageData::Open(sender) => sender,
            // If this actually happens we have a big problem.
            // Somehow we managed to close the websocket or
            // receive a message before opening the websocket.
            _ => unreachable!(),
        };

        let channel = wait.into_inner();

        Ok((
            Self {
                message_thread: handle,
                sender,
                protocol,
            },
            channel,
        ))
    }

    fn send_ws_frame(&self, frame: Vec<u8>) -> Result<(), ClientError<P>> {
        self.sender.send(frame)?;

        Ok(())
    }

    pub fn send_packet_ref(&self, packet: &ClientPacket) -> Result<(), ClientError<P>> {
        for frame in self
            .protocol
            .serialize_client(packet)
            .map_err(PacketSerializeError)?
        {
            self.send_ws_frame(frame)?;
        }

        Ok(())
    }

    pub fn send_packet<C>(&self, packet: C) -> Result<(), ClientError<P>>
    where
        C: Into<ClientPacket>,
    {
        self.send_packet_ref(&packet.into())
    }
}

impl ClientBase<ProtocolV5> {
    fn new(addr: String) -> Result<(Self, Receiver<ReceivedMessage>), ClientError<ProtocolV5>> {
        Self::build_with_protocol(addr, ProtocolV5 {})
    }
}

pub struct Client<S> {
    inner: S,
}

impl Client<()> {
    pub fn new<'a, S>(
        addr: &'a S,
    ) -> Result<
        Client<impl Stream<Item = ClientEvent<ProtocolV5>, Error = ClientError<ProtocolV5>>>,
        ClientError<ProtocolV5>,
    >
    where
        S: ToOwned<Owned = String> + ?Sized,
        String: Borrow<S>,
    {
        let s: String = addr.to_owned();

        Ok(Client {
            inner: build_client_stream(s)?,
        })
    }
}

impl<S, P> Client<S>
where
    P: Protocol + 'static,
    S: Stream<Item = ClientEvent<P>, Error = ClientError<P>>,
{
    pub fn from_stream(stream: S) -> Self {
        Self { inner: stream }
    }

    pub fn login_with_session_and_horizon(
        self,
        name: String,
        flag: String,
        session: Option<String>,
        horizon_x: u16,
        horizon_y: u16,
    ) -> Client<impl Stream<Item = ClientEvent<P>, Error = ClientError<P>>>
    where
        Self: Sized,
    {
        use protocol::client::Login;

        let packet = Login {
            name: name,
            flag: flag,
            session: session.unwrap_or("none".to_owned()),
            // Will get updated later
            protocol: 0,

            // These are usually ignored by the server
            horizon_x: horizon_x,
            horizon_y: horizon_y,
        };

        self.send_packet_with_cb(packet, |p, evt| p.protocol = evt.base.protocol.version())
    }

    pub fn login_with_session<N, X, F>(
        self,
        name: &N,
        flag: &F,
        session: X,
    ) -> Client<impl Stream<Item = ClientEvent<P>, Error = ClientError<P>>>
    where
        Self: Sized,
        N: ToOwned<Owned = String> + ?Sized,
        X: Into<Option<String>>,
        F: ToOwned<Owned = String> + ?Sized,
        String: Borrow<N> + Borrow<F>,
    {
        self.login_with_session_and_horizon(
            name.to_owned(),
            flag.to_owned(),
            session.into(),
            4500,
            4500,
        )
    }

    pub fn login<N, F>(
        self,
        name: &N,
        flag: &F,
    ) -> Client<impl Stream<Item = ClientEvent<P>, Error = ClientError<P>>>
    where
        Self: Sized,
        N: ToOwned<Owned = String> + ?Sized,
        F: ToOwned<Owned = String> + ?Sized,
        String: Borrow<N> + Borrow<F>,
    {
        self.login_with_session(name, flag, None)
    }

    pub fn send_packet<I>(
        self,
        packet: I,
    ) -> Client<impl Stream<Item = ClientEvent<P>, Error = ClientError<P>>>
    where
        I: Into<ClientPacket> + Send + 'static,
    {
        let mut packet = Some(packet);

        Client {
            inner: self.inner.inspect(move |evt| {
                if packet.is_none() {
                    return;
                }

                let p = mem::replace(&mut packet, None).unwrap();
                evt.base.send_packet(p).unwrap();
            }),
        }
    }

    pub fn send_packet_with_cb<I, F>(
        self,
        packet: I,
        cb: F,
    ) -> Client<impl Stream<Item = ClientEvent<P>, Error = ClientError<P>>>
    where
        I: Into<ClientPacket> + Send + 'static,
        F: FnOnce(&mut I, &ClientEvent<P>) -> (),
    {
        let mut packet = Some(packet);
        let mut cb = Some(cb);

        Client {
            inner: self.inner.inspect(move |evt| {
                if packet.is_none() {
                    return;
                }

                let mut p = mem::replace(&mut packet, None).unwrap();
                let cb = mem::replace(&mut cb, None).unwrap();

                cb(&mut p, evt);

                evt.base.send_packet(p).unwrap();
            }),
        }
    }

    pub fn wait(
        self,
        duration: Duration,
    ) -> Client<impl Stream<Item = ClientEvent<P>, Error = ClientError<P>>> {
        let mut end = None;

        Client {
            inner: self.inner.skip_while(move |evt| {
                if end.is_none() {
                    end = Some(Instant::now() + duration);
                }

                Ok(evt.inst < end.unwrap())
            }),
        }
    }

    pub fn chat<C>(
        self,
        message: C,
    ) -> Client<impl Stream<Item = ClientEvent<P>, Error = ClientError<P>>>
    where
        C: ToString,
    {
        use protocol::client::Chat;

        self.send_packet(Chat {
            text: message.to_string(),
        })
    }

    pub fn say<C>(
        self,
        message: C,
    ) -> Client<impl Stream<Item = ClientEvent<P>, Error = ClientError<P>>>
    where
        C: ToString,
    {
        use protocol::client::Say;

        self.send_packet(Say {
            text: message.to_string(),
        })
    }

    pub fn send_command<C, D>(
        self,
        command: C,
        data: D,
    ) -> Client<impl Stream<Item = ClientEvent<P>, Error = ClientError<P>>>
    where
        C: ToString,
        D: ToString,
    {
        use protocol::client::Command;

        self.send_packet(Command {
            com: command.to_string(),
            data: data.to_string(),
        })
    }

    pub fn change_flag<F>(
        self,
        flag: F,
    ) -> Client<impl Stream<Item = ClientEvent<P>, Error = ClientError<P>>>
    where
        F: ToString,
    {
        self.send_command("flag", flag)
    }

    pub fn enter_spectate(
        self
    )-> Client<impl Stream<Item = ClientEvent<P>, Error = ClientError<P>>> {
        self.send_command("spectate", "-1")
    }

    pub fn disconnect(self) -> impl Future<Item = (), Error = ClientError<P>> {
        self.inner
            .take_while(|_| {
                info!("Disconnected!");
                Ok(false)
            })
            .into_future()
            .map(|_| ())
            .map_err(|(e, _)| e)
    }

    pub fn set_key(
        self,
        keycode: KeyCode,
        state: bool,
    ) -> Client<impl Stream<Item = ClientEvent<P>, Error = ClientError<P>>> {
        use protocol::client::Key;

        let packet = Key {
            key: keycode,
            seq: 0,
            state,
        };

        self.send_packet_with_cb(packet, |p, evt| {
            let mut key_seq = evt.key_seq.lock().unwrap();

            p.seq = *key_seq;
            *key_seq += 1;
        })
    }

    pub fn press_key(
        self,
        keycode: KeyCode,
    ) -> Client<impl Stream<Item = ClientEvent<P>, Error = ClientError<P>>> {
        self.set_key(keycode, true)
    }
    pub fn release_key(
        self,
        keycode: KeyCode,
    ) -> Client<impl Stream<Item = ClientEvent<P>, Error = ClientError<P>>> {
        self.set_key(keycode, false)
    }

    pub fn switch_plane(
        self,
        plane: PlaneType
    ) -> Client<impl Stream<Item = ClientEvent<P>, Error = ClientError<P>>> {
        self.send_command("respawn", (plane as u8).to_string())
    }

    pub fn into_inner(self) -> S {
        self.inner
    }

    pub fn into_boxed(self) -> Client<Box<Stream<Item = ClientEvent<P>, Error = ClientError<P>> + Sync + Send>> 
    where
        S: Sync + Send + 'static
    {
        Client {
            inner: Box::new(self.inner)
        }
    }
}

impl<S, P> Stream for Client<S>
where
    P: Protocol + 'static,
    S: Stream<Item = ClientEvent<P>, Error = ClientError<P>>,
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Result<::futures::Async<Option<S::Item>>, S::Error> {
        self.inner.poll()
    }
}
