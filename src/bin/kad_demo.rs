use std::collections::HashMap;
use std::env;

use futures::Async;
use futures::future::poll_fn;
use futures::stream::Stream;

use libp2p::core::{Multiaddr, Swarm, PeerId};
use libp2p::secio::SecioKeyPair;
use libp2p::tokio_codec::{FramedRead, LinesCodec};

use tokio;

use golem_libp2p::behaviour::kad::{Kademlia, KademliaOut};

fn main() {
    let local_private_key = SecioKeyPair::ed25519_generated().unwrap();
    let local_peer_id = local_private_key.to_peer_id();
    println!("{}", local_peer_id.to_base58());

    let transport = libp2p::build_development_transport(local_private_key);
    let kademlia = Kademlia::new(local_peer_id.clone());
    let mut swarm = Swarm::new(transport, kademlia, local_peer_id.clone());

    let args: Vec<String> = env::args().collect();

    let port = args[1].parse::<u16>().unwrap();
    let addr: Multiaddr = format!("/ip4/127.0.0.1/tcp/{}", port).parse().unwrap();
    Swarm::listen_on(&mut swarm, addr.clone()).unwrap();

    if args.len() > 2 {
        let port = args[2].parse::<u16>().unwrap();
        let addr = format!("/ip4/127.0.0.1/tcp/{}", port).parse().unwrap();
        Swarm::dial_addr(&mut swarm, addr).unwrap();
    }

    let stdin = tokio_stdin_stdout::stdin(0);
    let mut framed_stdin = FramedRead::new(stdin, LinesCodec::new());

    let mut peer_addresses = HashMap::new();
    peer_addresses.insert(local_peer_id, addr);

    let mut event_counter = 0;

    tokio::run(poll_fn(move || {
        loop {
            match framed_stdin.poll().unwrap() {
                Async::Ready(Some(line)) => {
                    let decoded = bs58::decode(line).into_vec().unwrap();
                    let peer_id = PeerId::from_bytes(decoded).unwrap();

                    match peer_addresses.get(&peer_id) {
                        Some(addr) => println!("{} : {}", peer_id.to_base58(), addr),
                        None => swarm.find_node(peer_id)
                    }
                },
                Async::Ready(None) => {
                    eprintln!("{}", event_counter);
                    return Ok(Async::Ready(()))
                },
                Async::NotReady => break
            }
        }

        loop {
            match swarm.poll().unwrap() {
                Async::Ready(Some(KademliaOut::Discovered { peer_id, mut addresses, .. })) => {
                    event_counter += 1;
                    if addresses.len() > 0 && !peer_addresses.contains_key(&peer_id) {
                        let addr = addresses.remove(0);
                        println!("{} : {}", peer_id.to_base58(), addr);
                        peer_addresses.insert(peer_id, addr);
                    }
                }
                Async::Ready(None) | Async::NotReady => break,
                _ => { event_counter += 1 }
            }
        }

        Ok(Async::NotReady)
    }));

}
