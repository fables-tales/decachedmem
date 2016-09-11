extern crate tokio_core;
extern crate futures;
extern crate env_logger;

use std::env;
use std::net::SocketAddr;

use futures::Future;
use futures::stream::Stream;
use tokio_core::{Loop};

mod socket_stream;
mod crlf_delimited_stream;
mod memcached;

use socket_stream::SocketStream;
use crlf_delimited_stream::CarriageReturnLineFeedDelimitedStream;
use memcached::stream::MemcachedProtcolStream;

fn main() {
    env_logger::init().unwrap();
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    // Create the event loop that will drive this server
    let mut l = Loop::new().unwrap();
    let pin = l.pin();

    // Create a TCP listener which will listen for incoming connections
    let server = l.handle().tcp_listen(&addr);

    let done = server.and_then(move |socket| {
        // Once we've got the TCP listener, inform that we have it
        println!("Listening on: {}", addr);

        socket.incoming().for_each(move |(socket, _addr)| {
            let stream = SocketStream::new(socket);
            let crlf = CarriageReturnLineFeedDelimitedStream::new(Box::new(stream));
            let memcached = MemcachedProtcolStream::new(Box::new(crlf));
            let msg = memcached.for_each(|message| {
                println!("{:?}", message);
                Ok(())
            });

            pin.spawn(msg.map(|_| ()).map_err(|e|{
                println!("error: {}", e);
                ()
            }));
            Ok(())
        })
    });
    l.run(done).unwrap();
}
