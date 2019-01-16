use crate::{ClientError, ClientEvent};

use protocol::ClientPacket;

pub trait Handler {
    fn on_event(&mut self, evt: &ClientEvent) -> Result<Option<ClientPacket>, ClientError>;
}

impl Handler for () {
    fn on_event(&mut self, _: &ClientEvent) -> Result<Option<ClientPacket>, ClientError> {
        Ok(None)
    }
}
