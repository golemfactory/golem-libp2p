use std::env;

use futures::stream::Stream;

use libp2p::NetworkBehaviour;
use libp2p::core::swarm::{NetworkBehaviourEventProcess, Swarm};
use libp2p::secio::SecioKeyPair;

use tokio;
use tokio_io::{AsyncRead, AsyncWrite};

use void::Void;

use golem_libp2p::behaviour::reconnect::ReconnectBehaviour;
use golem_libp2p::behaviour::welcome::WelcomeBehaviour;

// Network behaviour composing welcome and re-connect behaviours
#[derive(NetworkBehaviour)]
struct MyBehaviour<TSubstream> {
    welcome: WelcomeBehaviour<TSubstream>,
    reconnect: ReconnectBehaviour<TSubstream>
}

impl<TSubstream> MyBehaviour<TSubstream> {
    fn new (welcome_message: String) -> Self {
        MyBehaviour {
            welcome: WelcomeBehaviour::new(welcome_message),
            reconnect: ReconnectBehaviour::new()
        }
    }
}

impl<TSubstream> NetworkBehaviourEventProcess<String> for MyBehaviour<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite
{
    fn inject_event(&mut self, event: String) {
        println!("Welcome message received: {}", event)
    }
}

impl<TSubstream> NetworkBehaviourEventProcess<Void> for MyBehaviour<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite
{
    fn inject_event(&mut self, _: Void) {}
}

fn main() {
    let local_private_key = SecioKeyPair::ed25519_generated().unwrap();
    let local_peer_id = local_private_key.to_peer_id();
    println!("My ID: {:?}", local_peer_id);

    let transport = libp2p::build_development_transport(local_private_key);
    let behaviour = MyBehaviour::new("Hello!".to_owned());
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
