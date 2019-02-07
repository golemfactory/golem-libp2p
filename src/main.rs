extern crate libp2p;

use std::env;
use std::marker::PhantomData;

use futures::Async;
use futures::stream::Stream;

use libp2p::core::PeerId;
use libp2p::core::multiaddr::Multiaddr;
use libp2p::core::nodes::raw_swarm::ConnectedPoint;
use libp2p::core::protocols_handler::DummyProtocolsHandler;
use libp2p::core::swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters, Swarm};
use libp2p::secio::SecioKeyPair;

use tokio;
use tokio_io::{AsyncRead, AsyncWrite};

use void::Void;


struct DummyBehaviour<TSubstream> {
    _marker: PhantomData<TSubstream>
}

impl<TSubstream> DummyBehaviour<TSubstream> {
    fn new() -> DummyBehaviour<TSubstream> {
        DummyBehaviour {_marker: PhantomData}
    }
}

impl<TSubstream> NetworkBehaviour for DummyBehaviour<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite
{
    type ProtocolsHandler = DummyProtocolsHandler<TSubstream>;
    type OutEvent = Void;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        DummyProtocolsHandler::default()
    }

    fn addresses_of_peer(&mut self, _: &PeerId) -> Vec<Multiaddr> {
        Vec::new()
    }

    fn inject_connected(&mut self, peer_id: PeerId, _: ConnectedPoint) {
        println!("Connected to peer: {:?}", peer_id);
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId, _: ConnectedPoint) {
        println!("Disconnected from peer: {:?}", peer_id);
    }

    fn inject_node_event(&mut self, _: PeerId, _: Void) { }

    fn poll(&mut self, _: &mut PollParameters) -> Async<NetworkBehaviourAction<Void, Void>> {
        Async::NotReady
    }
}

fn main() {
    let local_private_key = SecioKeyPair::ed25519_generated().unwrap();
    let local_peer_id = local_private_key.to_peer_id();
    println!("My ID: {:?}", local_peer_id);

    let transport = libp2p::build_development_transport(local_private_key);
    let behaviour = DummyBehaviour::new();
    let mut swarm = Swarm::new(transport, behaviour, local_peer_id);

    let args: Vec<String> = env::args().collect();

    let port = args[1].parse::<u16>().unwrap();
    let addr = format!("/ip4/127.0.0.1/tcp/{}", port).parse().unwrap();
    println!("Listening on {}...", addr);
    Swarm::listen_on(&mut swarm, addr).unwrap();

    if args.len() > 2 {
        let port = args[2].parse::<u16>().unwrap();
        let addr = format!("/ip4/127.0.0.1/tcp/{}", port).parse().unwrap();
        println!("Dialing {}...", addr);
        Swarm::dial_addr(&mut swarm, addr).unwrap();
    }

    tokio::run(swarm
        .map_err(|e| eprintln!("{}", e))
        .for_each(|_| Ok(()))
    );
}
