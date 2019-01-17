use protocol::ServerPacket;
use std::time::Instant;

pub enum ClientEvent {
    Frame(Instant),
    Packet(ServerPacket),
}
