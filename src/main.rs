#[macro_use]
extern crate tokio_core;
extern crate futures;
extern crate env_logger;

use std::env;
use std::net::SocketAddr;
use std::io::{self, Write};
use std::iter::repeat;

use futures::{Future, AndThen};
use futures::stream::{self, Stream, ForEach};
use tokio_core::io::{write_all, WriteAll, WriteHalf, Io};
use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;

mod socket_stream;
mod crlf_delimited_stream;
mod memcached;
mod copy_stream_to_write;
mod unpack;

use socket_stream::SocketStream;
use crlf_delimited_stream::CarriageReturnLineFeedDelimitedStream;
use memcached::stream::MemcachedProtcolStream;
use memcached::handler_stream::MemcachedHandlerStream;
use memcached::store::Store;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use copy_stream_to_write::CopyStreamToWrite;
use unpack::Unpack;

fn main() {
    env_logger::init().unwrap();
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:11211".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();


    // Create the event loop that will drive this server
    let mut l = Core::new().unwrap();
    let pin = l.handle();
    let store = Store::new();

    // Create a TCP listener which will listen for incoming connections
    let server = TcpListener::bind(&addr, &pin);

    match server {
        Ok(bound_socket) => {
            let store_stream = stream::iter(repeat(Ok(Arc::new(Mutex::new(store)))));
            let done = bound_socket.incoming().map_err(|_| ()).zip(store_stream).for_each(|((socket, _addr), store)| {
                let pair = futures::lazy(|| Ok(socket.split()));
                let foo = pair.and_then(|(read_half, write_half)| {
                    let stream = SocketStream::new(read_half);
                    let crlf = CarriageReturnLineFeedDelimitedStream::new(stream);
                    let memcached = MemcachedProtcolStream::new(crlf);
                    let handler = MemcachedHandlerStream::new(store, memcached);
                    let unpack = Unpack::new(handler);
                    let output = CopyStreamToWrite::new(unpack, write_half);
                    output.for_each(|_| Ok(()))
                });

                pin.spawn(foo.map(|_| ()).map_err(|e| {
                    println!("Done here!");
                    println!("{:?}", e);
                    ()
                }));
                Ok(())
            });

            l.run(done).unwrap();
        }
        Err(e) => {
            println!("binding failed: {}", e);
            return;
        },
    }
}
