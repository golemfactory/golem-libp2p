use std::collections::VecDeque;
use std::marker::PhantomData;

use futures::Async;

use libp2p::core::PeerId;
use libp2p::core::multiaddr::Multiaddr;
use libp2p::core::nodes::raw_swarm::ConnectedPoint;
use libp2p::core::protocols_handler::DummyProtocolsHandler;
use libp2p::core::swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters};

use tokio_io::{AsyncRead, AsyncWrite};

use void::Void;


// Network behaviour that re-connects to dialed nodes if the connection is dropped
pub struct ReconnectBehaviour<TSubstream> {
    network_actions: VecDeque<NetworkBehaviourAction<Void, Void>>,
    _marker: PhantomData<TSubstream>
}

impl<TSubstream> ReconnectBehaviour<TSubstream> {
    pub fn new() -> ReconnectBehaviour<TSubstream> {
        ReconnectBehaviour {
            network_actions: VecDeque::new(),
            _marker: PhantomData
        }
    }
}

impl<TSubstream> NetworkBehaviour for ReconnectBehaviour<TSubstream>
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

    fn inject_connected(&mut self, _: PeerId, _: ConnectedPoint) {
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId, connected_point: ConnectedPoint) {
        match connected_point {
            ConnectedPoint::Dialer { address } =>  {
                println!("Re-dialing peer {:?} on addr {:?}", peer_id, address);
                self.network_actions.push_back(NetworkBehaviourAction::DialAddress { address })
            },
            _ => {}
        }
    }

    fn inject_node_event(&mut self, _: PeerId, _: Void) { }

    fn poll(&mut self, _: &mut PollParameters) -> Async<NetworkBehaviourAction<Void, Void>> {
        match self.network_actions.pop_front() {
            Some(action) => Async::Ready(action),
            None => Async::NotReady
        }
    }
}
