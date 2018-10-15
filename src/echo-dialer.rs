extern crate bytes;
extern crate futures;
extern crate golem_libp2p;
extern crate libp2p;
extern crate tokio_codec;
extern crate tokio_current_thread;

use futures::sync::oneshot;
use futures::{Future, Sink, Stream};
use libp2p::SimpleProtocol;
use libp2p::core::Transport;
use libp2p::core::upgrade;
use libp2p::secio::SecioOutput;
use libp2p::tcp::TcpConfig;
use tokio_codec::{BytesCodec, Framed};

fn main() {

    let target_addr = golem_libp2p::get_address_from_arg();
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

    let (finished_tx, finished_rx) = oneshot::channel();
    let mut finished_tx = Some(finished_tx);

    let (swarm_controller, swarm_future) = libp2p::core::swarm(
        transport.clone(),
        |echo, _client_addr| {
            println!("Sending \"hello world\" to listener");
            let finished_tx = finished_tx.take();
            echo.send("hello world".into())
                // Then listening for one message from the remote.
                .and_then(|echo| {
                    echo.into_future().map_err(|(e, _)| e).map(|(n,_ )| n)
                })
                .and_then(move |message| {
                    println!("Received message from listener: {:?}", message.unwrap());
                    if let Some(finished_tx) = finished_tx {
                        finished_tx.send(()).unwrap();
                    }
                    Ok(())
                })
        },
    );

    swarm_controller
        .dial(target_addr.parse().expect("invalid multiaddr"), transport)
        .expect("unsupported multiaddr");

    let final_future = swarm_future
        .for_each(|_| Ok(()))
        .select(finished_rx.map_err(|_| unreachable!()))
        .map(|_| ())
        .map_err(|(err, _)| err);
    tokio_current_thread::block_on_all(final_future).unwrap();
}
