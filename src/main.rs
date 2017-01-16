extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;

use std::io;
use std::str;
use tokio_core::io::{Io, Framed};
use tokio_core::io::{Codec, EasyBuf};
use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;
use tokio_service::{Service, NewService};
use tokio_proto::TcpServer;
use futures::{future, Future, Stream, Sink, BoxFuture};

mod memcached;
mod codec;
mod protocol;

fn main() {}
