use std::env;

use futures::stream::Stream;

use libp2p::core::swarm::Swarm;
use libp2p::secio::SecioKeyPair;

use tokio;

use golem_libp2p::behaviour::welcome::WelcomeBehaviour;

fn main() {
    let local_private_key = SecioKeyPair::ed25519_generated().unwrap();
    let local_peer_id = local_private_key.to_peer_id();
    println!("My ID: {:?}", local_peer_id);

    let transport = libp2p::build_development_transport(local_private_key);
    let behaviour = WelcomeBehaviour::new("Hello!".to_owned());
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
        .for_each(|message| {
            println!("Received message: {}", message);
            Ok(())
        })
    );
}
