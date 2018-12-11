use std::time::Instant;

use protocol::ClientPacket;

pub enum ChannelEvent {
    Frame { time: Instant },
    Packet { data: ClientPacket },
    Close,
}
