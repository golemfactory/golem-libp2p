use std::convert::From;
use std::io;
use std::iter;
use std::string::FromUtf8Error;

use libp2p::core::{InboundUpgrade, OutboundUpgrade, UpgradeInfo};
use libp2p::core::protocols_handler::OneShotHandler;
use libp2p::core::upgrade::{ReadOneError, ReadOneThen, read_one_then, WriteOne, write_one};

use tokio_io::{AsyncRead, AsyncWrite};

const MAX_MESSAGE_SIZE: usize = 10240;  // bytes


// Simple protocol that allows to send and receive a text message
#[derive(Debug)]
pub enum SimpleProtocolError {
    ReadError(ReadOneError),
    ParseError(FromUtf8Error),
}

impl From<ReadOneError> for SimpleProtocolError {
    fn from(err: ReadOneError) -> Self {
        SimpleProtocolError::ReadError(err)
    }
}

impl From<FromUtf8Error> for SimpleProtocolError {
    fn from(err: FromUtf8Error) -> Self {
        SimpleProtocolError::ParseError(err)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SimpleProtocolConfig {}

impl SimpleProtocolConfig {
    pub fn new() -> SimpleProtocolConfig {
        SimpleProtocolConfig {}
    }
}

impl UpgradeInfo for SimpleProtocolConfig {
    type Info = &'static [u8];
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(b"/golem/1.0.0")
    }
}

impl<TSocket> InboundUpgrade<TSocket> for SimpleProtocolConfig
where
    TSocket: AsyncRead
{
    type Output = String;
    type Error = SimpleProtocolError;
    type Future = ReadOneThen<TSocket, fn(Vec<u8>) -> Result<String, SimpleProtocolError>>;

    fn upgrade_inbound(self, socket: TSocket, _: Self::Info) -> Self::Future {
        read_one_then(socket, MAX_MESSAGE_SIZE, |bytes| {
            let message = String::from_utf8(bytes)?;
            Ok(message)
        })
    }
}

#[derive(Debug, Clone)]
pub struct SimpleProtocolMessage(String);

impl SimpleProtocolMessage {
    pub fn new(str: String) -> Self {
        SimpleProtocolMessage(str)
    }
}

impl UpgradeInfo for SimpleProtocolMessage {
    type Info = &'static [u8];
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(b"/golem/1.0.0")
    }
}

impl<TSocket> OutboundUpgrade<TSocket> for SimpleProtocolMessage
where
    TSocket: AsyncWrite
{
    type Output = ();
    type Error = io::Error;
    type Future = WriteOne<TSocket>;

    fn upgrade_outbound(self, socket: TSocket, _: Self::Info) -> Self::Future {
        let bytes = self.0.into_bytes();
        write_one(socket, bytes)
    }
}

pub enum SimpleProtocolEvent {
    Received(String),
    Sent
}

impl From<String> for SimpleProtocolEvent {
    fn from(str: String) -> Self {
        SimpleProtocolEvent::Received(str)
    }
}

impl From<()> for SimpleProtocolEvent {
    fn from(_: ()) -> Self {
        SimpleProtocolEvent::Sent
    }
}

pub type SimpleProtocolsHandler<TSubstream> = OneShotHandler<TSubstream, SimpleProtocolConfig, SimpleProtocolMessage, SimpleProtocolEvent>;
