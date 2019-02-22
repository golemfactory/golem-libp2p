use std::collections::VecDeque;
use std::marker::PhantomData;

use futures::Async;

use libp2p::core::{PeerId};
use libp2p::core::multiaddr::Multiaddr;
use libp2p::core::nodes::raw_swarm::ConnectedPoint;
use libp2p::core::swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters};

use tokio_io::{AsyncRead, AsyncWrite};

use super::super::protocol::simple::{SimpleProtocolEvent, SimpleProtocolsHandler, SimpleProtocolMessage};


// Network behaviour that sends a single welcome message to every node which connects
pub struct WelcomeBehaviour<TSubstream> {
    welcome_message: String,
    network_actions: VecDeque<NetworkBehaviourAction<SimpleProtocolMessage, String>>,
    _marker: PhantomData<TSubstream>
}

impl<TSubstream> WelcomeBehaviour<TSubstream> {
    pub fn new(welcome_message: String) -> Self {
        WelcomeBehaviour {
            welcome_message,
            network_actions: VecDeque::new(),
            _marker: PhantomData
        }
    }
}

impl<TSubstream> NetworkBehaviour for WelcomeBehaviour<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite
{
    type ProtocolsHandler = SimpleProtocolsHandler<TSubstream>;
    type OutEvent = String;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        Self::ProtocolsHandler::default()
    }

    fn addresses_of_peer(&mut self, _: &PeerId) -> Vec<Multiaddr> {
        Vec::new()
    }

    fn inject_connected(&mut self, peer_id: PeerId, _: ConnectedPoint) {
        println!("Connected to peer: {:?}", peer_id);

        self.network_actions.push_back(NetworkBehaviourAction::SendEvent {
            peer_id,
            event: SimpleProtocolMessage::new(self.welcome_message.clone())
        })
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId, _: ConnectedPoint) {
        println!("Disconnected from peer: {:?}", peer_id);
    }

    fn inject_node_event(&mut self, _: PeerId, event: SimpleProtocolEvent) {
        match event {
            SimpleProtocolEvent::Received(message) => self.network_actions.push_back(
                NetworkBehaviourAction::GenerateEvent(message)),
            SimpleProtocolEvent::Sent => {}
        }
    }

    fn poll(&mut self, _: &mut PollParameters) -> Async<NetworkBehaviourAction<SimpleProtocolMessage, String>> {
        match self.network_actions.pop_front() {
            Some(action) => Async::Ready(action),
            None => Async::NotReady
        }
    }
}
