use ws::{self, CloseCode, Error as WsError, ErrorKind, Handler, Handshake, Message, Result};

use futures::sync::mpsc::UnboundedSender as Sender;

use std::time::Instant;

use received_message::*;

use self::ReceivedMessageData::*;

struct MessageHandler {
    channel: Sender<ReceivedMessage>,
    sender: ws::Sender,
}

impl Handler for MessageHandler {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        let result = self.channel.unbounded_send(ReceivedMessage {
            time: Instant::now(),
            data: Open(self.sender.clone()),
        });

        result.map_err(|e| WsError {
            details: e.to_string().into(),
            kind: ErrorKind::Custom(e.into()),
        })
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let result = match msg {
            Message::Binary(v) => self.channel.unbounded_send(ReceivedMessage {
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

        result.map_err(|e| WsError {
            details: e.to_string().into(),
            kind: ErrorKind::Custom(e.into()),
        })
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        info!("sent close");
        self.channel
            .unbounded_send(ReceivedMessage {
                time: Instant::now(),
                data: Close,
            })
            .unwrap();
    }
}

pub(crate) fn websocket_runner(addr: String, channel: Sender<ReceivedMessage>) {
    info!("Starting websocket connection to {}", addr);

    let result = ws::connect(addr, move |out| MessageHandler {
        channel: channel.clone(),
        sender: out,
    });

    if let Err(e) = result {
        error!("Connection failed with error {}", e);
    }
}
