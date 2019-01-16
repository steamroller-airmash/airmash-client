use airmash_protocol_v5::{DeserializeError, SerializeError};
use tokio::timer::Error as TimerError;
use tungstenite::Error as WsError;

use std::error::Error;
use std::fmt::{Display, Error as FmtError, Formatter};

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug)]
pub enum ClientError {
    WebSocket(WsError),
    Serialize(SerializeError),
    Deserialize(DeserializeError),
    Timer(TimerError),
    InvalidWsFrame(String),
    Custom(Box<Error + Send + 'static>),
}

impl From<WsError> for ClientError {
    fn from(e: WsError) -> Self {
        ClientError::WebSocket(e)
    }
}

impl From<SerializeError> for ClientError {
    fn from(e: SerializeError) -> Self {
        ClientError::Serialize(e)
    }
}

impl From<DeserializeError> for ClientError {
    fn from(e: DeserializeError) -> Self {
        ClientError::Deserialize(e)
    }
}

impl From<TimerError> for ClientError {
    fn from(e: TimerError) -> Self {
        ClientError::Timer(e)
    }
}

impl Display for ClientError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        use self::ClientError::*;
        match self {
            WebSocket(e) => write!(fmt, "WebSocket({})", e),
            Serialize(e) => write!(fmt, "Serialize({})", e),
            Deserialize(e) => write!(fmt, "Deserialize({})", e),
            Timer(e) => write!(fmt, "Timer({})", e),
            InvalidWsFrame(desc) => write!(fmt, "InvalidWsFrame({})", desc),
            Custom(e) => write!(fmt, "Custom({})", e),
        }
    }
}

impl Error for ClientError {}
