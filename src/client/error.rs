use airmash_protocol::v5::Error as ProtocolError;
use tungstenite::Error as WsError;

use std::error::Error;
use std::fmt::{Display, Error as FmtError, Formatter};

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug)]
pub enum ClientError {
    WebSocket(WsError),
    Protocol(ProtocolError),
    InvalidWsFrame(String),
}

impl From<WsError> for ClientError {
    fn from(e: WsError) -> Self {
        ClientError::WebSocket(e)
    }
}

impl From<ProtocolError> for ClientError {
    fn from(e: ProtocolError) -> Self {
        ClientError::Protocol(e)
    }
}

impl Display for ClientError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        use self::ClientError::*;
        match self {
            WebSocket(e) => write!(fmt, "WebSocket({})", e),
            Protocol(e) => write!(fmt, "Protocol({})", e),
            InvalidWsFrame(desc) => write!(fmt, "InvalidWsFrame({})", desc),
        }
    }
}

impl Error for ClientError {}
