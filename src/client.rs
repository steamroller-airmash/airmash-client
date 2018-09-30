use protocol::{ClientPacket, Protocol};
use protocol_v5::ProtocolV5;

use std::borrow::Borrow;
use std::error::Error;
use std::mem;
use std::sync::mpsc::{channel, Receiver};
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, Instant};

use gamestate::GameState;
use message_handler::*;
use received_message::{ReceivedMessage, ReceivedMessageData};

use ws::{CloseCode, Sender};

const FRAME_TIME: Duration = Duration::from_nanos(16666667);

pub struct Client<P: Protocol> {
    message_thread: Option<JoinHandle<()>>,
    packets: Receiver<ReceivedMessage>,
    sender: Sender,
    last_update: Instant,
    protocol: P,
    closed: bool,
    pub state: GameState,
}

impl<P: Protocol> Client<P>
where
    P::SerializeError: 'static,
    P::DeserializeError: 'static,
{
    fn create_event_thread(addr: String) -> (JoinHandle<()>, Receiver<ReceivedMessage>) {
        let (message_send, message_recv) = channel();
        let handle = spawn(move || websocket_runner(addr, message_send));

        (handle, message_recv)
    }

    pub fn with_protocol<S>(addr: S, protocol: P) -> Result<Self, Box<Error>>
    where
        S: ToOwned<Owned = String>,
        String: Borrow<S>,
    {
        use self::ReceivedMessageData::Open;

        let (handle, channel) = Self::create_event_thread(addr.to_owned());

        let sender = match channel.recv()?.data {
            Open(sender) => sender,
            // If this actually happens we have a big problem.
            // Somehow we managed to close the websocket or
            // receive a message before opening the websocket.
            _ => unreachable!(),
        };

        Ok(Self {
            message_thread: Some(handle),
            packets: channel,
            last_update: Instant::now(),
            sender: sender,
            protocol,
            closed: false,
            state: GameState::default(),
        })
    }

    fn update_state_once(&mut self) -> Result<(), Box<Error>> {
        let frame_end = self.last_update + FRAME_TIME;
        let iter = self.packets.try_iter().filter(|x| x.time < frame_end);

        for msg in iter {
            if let Ok(packet) = msg.as_packet(&self.protocol) {
                self.state.update_state(&packet);
            }
        }

        self.state.update_frame(frame_end);

        self.last_update = frame_end;

        Ok(())
    }

    pub fn update_state<'a>(&'a mut self) -> Result<&'a mut Self, Box<Error>> {
        let now = Instant::now();

        while now - self.last_update > FRAME_TIME {
            self.update_state_once()?;
        }

        Ok(self)
    }

    fn send_ws_frame<'a>(&'a mut self, frame: Vec<u8>) -> Result<(), Box<Error>> {
        self.sender.send(frame)?;

        Ok(())
    }

    pub fn send_packet_ref<'a>(
        &'a mut self,
        packet: &ClientPacket,
    ) -> Result<&'a mut Self, Box<Error>> {
        for frame in self.protocol.serialize_client(packet)? {
            self.send_ws_frame(frame)?;
        }

        Ok(self)
    }

    pub fn send_packet<'a, C>(&'a mut self, packet: C) -> Result<&'a mut Self, Box<Error>>
    where
        C: Into<ClientPacket>,
    {
        self.send_packet_ref(&packet.into())
    }

    pub fn login_with_session_and_horizon<'a>(
        &'a mut self,
        name: String,
        flag: String,
        session: Option<String>,
        horizon_x: u16,
        horizon_y: u16,
    ) -> Result<&'a mut Self, Box<Error>> {
        use protocol::client::Login;

        let packet = Login {
            name: name.to_owned(),
            flag: flag.to_owned(),
            session: session.unwrap_or("none".to_owned()),
            protocol: self.protocol.version(),

            // These are usually ignored by the server
            horizon_x: horizon_x,
            horizon_y: horizon_y,
        };

        self.send_packet(packet)
    }

    pub fn login_with_session<'a, N, S, F>(
        &'a mut self,
        name: N,
        flag: F,
        session: S,
    ) -> Result<&'a mut Self, Box<Error>>
    where
        N: ToOwned<Owned = String>,
        S: Into<Option<String>>,
        F: ToOwned<Owned = String>,
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

    pub fn login<'a, N, F>(&'a mut self, name: N, flag: F) -> Result<&'a mut Self, Box<Error>>
    where
        N: ToOwned<Owned = String>,
        F: ToOwned<Owned = String>,
        String: Borrow<N> + Borrow<F>,
    {
        self.login_with_session(name.to_owned(), flag.to_owned(), None)
    }

    pub fn disconnect<'a>(&'a mut self) -> Result<&'a mut Self, Box<Error>> {
        self.update_state()?;

        self.sender
            .close(CloseCode::Normal)
            .expect("Failed to close connection!");
        self.closed = true;

        Ok(self)
    }
}

impl Client<ProtocolV5> {
    pub fn new<S>(addr: S) -> Result<Self, Box<Error>>
    where
        S: ToOwned<Owned = String>,
        String: Borrow<S>,
    {
        Self::with_protocol(addr, ProtocolV5)
    }
}

impl<P: Protocol> Drop for Client<P> {
    fn drop(&mut self) {
        if !self.closed {
            self.sender
                .close(CloseCode::Normal)
                .expect("Failed to send close message!");
        }

        mem::replace(&mut self.message_thread, None).map(|x| x.join().unwrap());
    }
}
