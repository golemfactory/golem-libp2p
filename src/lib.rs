extern crate ethkey;
extern crate libp2p;

use std::env;
use std::io::Error as IoError;
use libp2p::SimpleProtocol;
use libp2p::secio::{SecioConfig, SecioKeyPair};


pub fn generate_keypair() -> SecioKeyPair {
	SecioKeyPair::secp256k1_generated().expect("Wrongly generated keys")
}

pub fn get_secio_upgrade() -> SecioConfig {
	SecioConfig::new(generate_keypair())
}

