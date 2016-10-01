use futures::stream::Stream;
use futures::{Poll, Async};
use std::io;
use std::error::Error;

use memcached::parser::MemcachedParseStateMachine;
use memcached::types::*;

fn other_io_error<T>(error: T) -> io::Error
    where T: Error + Sized
{
    io::Error::new(io::ErrorKind::Other, error.description())
}


pub struct MemcachedProtcolStream<T: Stream<Item = Vec<u8>, Error = io::Error>> {
    stream: T,
    parser: MemcachedParseStateMachine,
}

impl<T: Stream<Item = Vec<u8>, Error = io::Error>> MemcachedProtcolStream<T> {
    pub fn new(stream: T) -> MemcachedProtcolStream<T> {
        MemcachedProtcolStream {
            stream: stream,
            parser: MemcachedParseStateMachine::new(),
        }
    }
}

impl<T: Stream<Item = Vec<u8>, Error = io::Error>> Stream for MemcachedProtcolStream<T> {
    type Item = MemcachedFrame;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            let poll = try!(self.stream.poll());
            match poll {
                Async::Ready(Some(x)) => {
                    let frame = try!(self.parser.add_bytes(&x).map_err(|e| other_io_error(e)));

                    match frame {
                        Some(x) => return Ok(Async::Ready(Some(x))),
                        None => {}
                    }
                }
                Async::NotReady |
                Async::Ready(None) => return Ok(Async::NotReady),
            };
        }
    }
}
