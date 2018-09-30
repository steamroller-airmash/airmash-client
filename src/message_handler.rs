use ws::{self, CloseCode, Handler, Handshake, Message, Result};

use std::sync::mpsc::Sender;
use std::time::Instant;

use error::AbortError;
use received_message::*;

use self::ReceivedMessageData::*;

struct MessageHandler {
    channel: Sender<ReceivedMessage>,
    sender: ws::Sender,
}

impl Handler for MessageHandler {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        let result = self.channel.send(ReceivedMessage {
            time: Instant::now(),
            data: Open(self.sender.clone()),
        });

        result.map_err(|_| AbortError.into())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let result = match msg {
            Message::Binary(v) => self.channel.send(ReceivedMessage {
                time: Instant::now(),
                data: Binary(v),
            }),
            Message::Text(msg) => {
                warn!(
                    target: "server-error",
                    "The server sent a text packet! This is always an invalid packet for airmash given current protocols. Packet Data:\n{}",
                    msg
                );

                Ok(())
            }
        };

        result.map_err(|_| AbortError.into())
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        self.channel
            .send(ReceivedMessage {
                time: Instant::now(),
                data: Close,
            })
            .ok();
    }
}

pub(crate) fn websocket_runner(addr: String, channel: Sender<ReceivedMessage>) {
    ws::connect(addr, move |out| MessageHandler {
        channel: channel.clone(),
        sender: out,
    }).ok();
}
