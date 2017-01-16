extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;

use std::str;
use std::sync::{Arc, Mutex};
use tokio_proto::TcpServer;

mod memcached;
mod codec;
mod protocol;
mod service;

use protocol::MemcachedProto;
use service::MemcachedService;

fn main() {
    let addr = "0.0.0.0:11211".parse().unwrap();

    // The builder requires a protocol and an address
    let server = TcpServer::new(MemcachedProto::new(), addr);

    let store = Arc::new(Mutex::new(memcached::Store::new()));

    // We provide a way to *instantiate* the service for each new
    // connection; here, we just immediately return a new instance.
    server.serve(move || {
        println!("here");
        Ok(MemcachedService::new(store.clone()))
    });
}
