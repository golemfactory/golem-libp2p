extern crate bytes;
extern crate futures;
extern crate golem_libp2p;
extern crate libp2p;
extern crate tokio_codec;
extern crate tokio_current_thread;

use futures::future::{loop_fn, Future, IntoFuture, Loop};
use futures::{Sink, Stream};
use libp2p::core::Transport;
use libp2p::core::upgrade;
use libp2p::secio::SecioOutput;
use libp2p::tcp::TcpConfig;

fn main() {
    let listen_addr = golem_libp2p::get_address_from_arg();
    let proto = golem_libp2p::get_golem_protocol();
 
    let transport = TcpConfig::new()
        .with_upgrade(
			upgrade::map(golem_libp2p::get_secio_upgrade(), 
				|out: SecioOutput<_>| out.stream)
        )
        .with_upgrade(libp2p::mplex::MplexConfig::new())
        .map(|val, _| ((), val))
        .into_connection_reuse()
        .map(|((), val), _| val)
		.with_upgrade(proto);

    
	let (swarm_controller, swarm_future) = libp2p::core::swarm(
        transport.clone(),
        |socket, _client_addr| {
            println!("Successfully negotiated protocol");

            loop_fn(socket, move |socket| {
                socket
                    .into_future()
                    .map_err(|(e, _)| e)
                    .and_then(move |(msg, rest)| {
                        if let Some(msg) = msg {
                            println!(
                                "Received a message: {:?}\n => Sending back \
                                 identical message to remote", msg
                            );
                            Box::new(rest.send(msg.freeze()).map(|m| Loop::Continue(m)))
                                as Box<Future<Item = _, Error = _>>
                        } else {
                            println!("Received EOF\n => Dropping connection");
                            Box::new(Ok(Loop::Break(())).into_future())
                                as Box<Future<Item = _, Error = _>>
                        }
                    })
            })
        },
    );

    let address = swarm_controller
        .listen_on(listen_addr.parse().expect("invalid multiaddr"))
        .expect("unsupported multiaddr");
    println!("Now listening on {:?}", address);

	let final_future = swarm_future
		.for_each(|_| Ok(()));	

    tokio_current_thread::block_on_all(final_future).unwrap();
}
