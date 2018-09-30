use protocol::Protocol;

use ws::{Error as WsError, ErrorKind};

use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Error as FmtError, Formatter};

#[derive(Copy, Clone, Debug)]
pub enum PacketDeserializeError<P: Protocol> {
    NotABinaryPacket,
    InvalidPacket(P::DeserializeError),
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct AbortError;

#[derive(Copy, Clone, Debug)]
pub struct ConnectionClosedError;

impl Display for AbortError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        write!(fmt, "Aborted event loop")
    }
}

impl Error for AbortError {}

impl Display for ConnectionClosedError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        write!(fmt, "The websocket connection to the server is not open")
    }
}

impl Error for ConnectionClosedError {}

impl From<AbortError> for WsError {
    fn from(_: AbortError) -> Self {
        WsError {
            kind: ErrorKind::Custom(Box::new(AbortError)),
            details: Cow::Borrowed(
                "The channel was closed from the client loop without notifying the event loop.",
            ),
        }
    }
}
