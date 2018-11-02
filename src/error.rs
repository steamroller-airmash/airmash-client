use protocol::Protocol;

use ws::{Error as WsError, ErrorKind};

use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Error as FmtError, Formatter};

use tokio::timer::Error as TimerError;

#[derive(Derivative)]
#[derivative(Debug(bound = "P::SerializeError: Debug"))]
pub enum ClientError<P>
where
    P: Protocol,
{
    Deserialization(PacketDeserializeError<P>),
    Serialization(PacketSerializeError<P>),
    Timer(TimerError),
    WsError(WsError),
    Other
}

impl<P> From<AbortError> for ClientError<P>
where
    P: Protocol
{
    fn from(_: AbortError) -> Self {
         ClientError::Other
    }
}

impl<P> From<TimerError> for ClientError<P>
where
    P: Protocol,
{
    fn from(e: TimerError) -> Self {
        ClientError::Timer(e)
    }
}

impl<P> From<PacketSerializeError<P>> for ClientError<P>
where
    P: Protocol,
{
    fn from(e: PacketSerializeError<P>) -> Self {
        ClientError::Serialization(e)
    }
}

impl<P> From<WsError> for ClientError<P>
where
    P: Protocol,
{
    fn from(e: WsError) -> Self {
        ClientError::WsError(e)
    }
}

impl<P: Protocol> Display for ClientError<P> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        write!(fmt, "{:?}", self)
    }
}

impl<P: Protocol> Error for ClientError<P> {}

#[derive(Copy, Clone, Derivative)]
#[derivative(Debug(bound = "P::DeserializeError: Debug"))]
pub enum PacketDeserializeError<P: Protocol> {
    NotABinaryPacket,
    InvalidPacket(P::DeserializeError),
}

#[derive(Copy, Clone, Derivative)]
#[derivative(Debug(bound = "P::SerializeError: Debug"))]
pub struct PacketSerializeError<P: Protocol>(pub P::SerializeError);

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
