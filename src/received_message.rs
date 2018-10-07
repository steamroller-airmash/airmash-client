use ws::Sender;

use error::PacketDeserializeError;
use protocol::{Protocol, ServerPacket};

use std::time::Instant;

pub(crate) struct ReceivedMessage {
    pub data: ReceivedMessageData,
    pub time: Instant,
}

pub(crate) enum ReceivedMessageData {
    Binary(Vec<u8>),
    Open(Sender),
    Close,
}

impl ReceivedMessage {
    pub fn is_close(&self) -> bool {
        match self.data {
            ReceivedMessageData::Close => true,
            _ => false
        }
    }

    pub fn as_packet<P>(&self, protocol: &P) -> Result<ServerPacket, PacketDeserializeError<P>>
    where
        P: Protocol,
    {
        use self::PacketDeserializeError::*;
        use self::ReceivedMessageData::Binary;

        match self.data {
            Binary(ref v) => Ok(protocol.deserialize_server(&v).map_err(InvalidPacket)?),
            _ => Err(NotABinaryPacket),
        }
    }
}
