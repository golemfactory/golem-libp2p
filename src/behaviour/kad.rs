use std::ops::{Deref, DerefMut};

use futures::Async;

use libp2p::{PeerId, kad, Multiaddr};
use libp2p::core::protocols_handler::{ProtocolsHandler};
use libp2p::core::swarm::{ConnectedPoint, NetworkBehaviour, NetworkBehaviourAction, PollParameters};
pub use libp2p::kad::KademliaOut;

use tokio::prelude::{AsyncRead, AsyncWrite};

pub struct Kademlia<TSubstream> {
    local_peer_id: PeerId,
    kademlia: kad::Kademlia<TSubstream>
}

impl <TSubstream> Kademlia<TSubstream> {
    pub fn new(local_peer_id: PeerId) -> Self {
        Kademlia {
            local_peer_id: local_peer_id.clone(),
            kademlia: kad::Kademlia::new(local_peer_id)
        }
    }
}

impl<TSubstream> Deref for Kademlia<TSubstream> {
    type Target = kad::Kademlia<TSubstream>;

    fn deref(&self) -> &Self::Target {
        &self.kademlia
    }
}

impl<TSubstream> DerefMut for Kademlia<TSubstream> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kademlia
    }
}

impl<TSubstream> NetworkBehaviour for Kademlia<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite
{
    type ProtocolsHandler = <kad::Kademlia<TSubstream> as NetworkBehaviour>::ProtocolsHandler;
    type OutEvent = <kad::Kademlia<TSubstream> as NetworkBehaviour>::OutEvent;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        self.kademlia.new_handler()
    }

    fn addresses_of_peer(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
        self.kademlia.addresses_of_peer(peer_id)
    }

    fn inject_connected(&mut self, peer_id: PeerId, endpoint: ConnectedPoint) {
        self.kademlia.inject_connected(peer_id.clone(), endpoint.clone());

        // ********************************************************
        // *** THESE ARE THE ONLY IMPORTANT LINES IN THIS FILE  ***
        // ***          THE REST IS JUST BOILERPLATE            ***
        // *** WITHOUT THESE INITIAL QUERIES NODES WOULD NOT BE ***
        // ***           ABLE TO LOCATE EACH OTHER              ***
        // ********************************************************
        match endpoint {
            ConnectedPoint::Listener {..} => self.kademlia.find_node(peer_id),
            ConnectedPoint::Dialer {..} => self.kademlia.find_node(self.local_peer_id.clone()),
        }

    }

    fn inject_disconnected(&mut self, peer_id: &PeerId, endpoint: ConnectedPoint) {
        self.kademlia.inject_disconnected(peer_id, endpoint)
    }

    fn inject_replaced(&mut self, peer_id: PeerId, endpoint: ConnectedPoint, new_endpoint: ConnectedPoint) {
        self.kademlia.inject_replaced(peer_id, endpoint, new_endpoint)
    }

    fn inject_node_event(&mut self, source: PeerId, event: <Self::ProtocolsHandler as ProtocolsHandler>::OutEvent) {
        self.kademlia.inject_node_event(source, event)
    }

    fn poll(&mut self, parameters: &mut PollParameters) -> Async<
        NetworkBehaviourAction<
            <Self::ProtocolsHandler as ProtocolsHandler>::InEvent, Self::OutEvent>>
    {
        self.kademlia.poll(parameters)
    }
}
