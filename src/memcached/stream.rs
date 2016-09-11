use futures::stream::Stream;
use futures::{Poll, Async};
use std::io;
use std::error::Error;

use memcached::parser::MemcachedParseStateMachine;
use memcached::types::*;

fn other_io_error<T>(error: T) -> io::Error where T: Error + Sized {
    io::Error::new(io::ErrorKind::Other, error.description())
}


pub struct MemcachedProtcolStream {
    stream: Box<Stream<Item=Box<Vec<u8>>, Error=io::Error>>,
    parser: MemcachedParseStateMachine,
}

impl MemcachedProtcolStream {
    pub fn new(stream: Box<Stream<Item=Box<Vec<u8>>, Error=io::Error>>) -> MemcachedProtcolStream {
        MemcachedProtcolStream {
            stream: stream,
            parser: MemcachedParseStateMachine::new(),
        }
    }
}

impl Stream for MemcachedProtcolStream {
    type Item=MemcachedFrame;
    type Error=io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            let poll = self.stream.poll();
            match poll {
                Ok(Async::Ready(Some(x))) => {
                    println!("{:?}", x);
                    let frame = try!(self.parser.add_bytes(&x).map_err(|e| other_io_error(e)));

                    match frame {
                        Some(x) => return Ok(Async::Ready(Some(x))),
                        None => {},
                    }
                },
                Ok(Async::Ready(None)) => {
                    return Ok(Async::Ready(None))
                },
                Ok(Async::NotReady) => {
                    return Ok(Async::NotReady)
                },
                Err(e) => {
                    return Err(e)
                }
            };
        }
    }
}


