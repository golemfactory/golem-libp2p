extern crate libp2p;
extern crate tokio_codec;
extern crate tokio_io;

use std::env;
use std::io::Error as IoError;
use libp2p::SimpleProtocol;
use libp2p::secio::{SecioConfig, SecioKeyPair};
use tokio_codec::{BytesCodec, Framed};
use tokio_io::{AsyncRead, AsyncWrite};

pub const DEFAULT_ADDRESS: &str = "/ip4/127.0.0.1/tcp/10333";
pub const PROTOCOL: &str = "/echo/1.0.0";


pub fn get_address_from_arg() -> String {
	env::args().nth(1).unwrap_or(DEFAULT_ADDRESS.to_owned())
}

pub fn generate_keypair() -> SecioKeyPair {
	SecioKeyPair::secp256k1_generated().expect("Wrongly generated keys")
}

pub fn get_secio_upgrade() -> SecioConfig {
	SecioConfig::new(generate_keypair())
}

pub fn get_protocol<TSocket>(name: String) -> 
	SimpleProtocol<impl Fn(TSocket) -> 
		Result<Framed<TSocket, BytesCodec>, IoError>> 
	where TSocket: AsyncRead + AsyncWrite
{
	SimpleProtocol::new(name, |socket| {
		Ok(Framed::new(socket, BytesCodec::new()))
	})
}

pub fn get_golem_protocol<TSocket>() -> 
	SimpleProtocol<impl Fn(TSocket) -> 
		Result<Framed<TSocket, BytesCodec>, IoError>>
	where TSocket: AsyncRead + AsyncWrite
{
	get_protocol(PROTOCOL.to_string())
}
