use airmash_protocol_v5::{DeserializeError, SerializeError};
use tungstenite::Error;

pub enum ClientError {
    WebSocket(Error),
    Serialize(SerializeError),
    Deserialize(DeserializeError),
    InvalidWsFrame,
}

impl From<Error> for ClientError {
    fn from(e: Error) -> Self {
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
