use std::time::Instant;

use crate::protocol::ClientPacket;

pub enum ChannelEvent {
    Frame { time: Instant },
    Packet { data: ClientPacket },
    Close,
}
