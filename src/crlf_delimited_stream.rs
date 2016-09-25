use std::{io, mem};
use futures::stream::Stream;
use std::str;
use futures::{Poll, Async};

pub type BoxedReadStream = Box<Stream<Item=u8, Error=io::Error>>;

pub struct CarriageReturnLineFeedDelimitedStream {
    stream: BoxedReadStream,
    build: Vec<u8>,
}

impl CarriageReturnLineFeedDelimitedStream {
    pub fn new(stream: BoxedReadStream) -> CarriageReturnLineFeedDelimitedStream {
        CarriageReturnLineFeedDelimitedStream {
            stream: stream,
            build: vec!(),
        }
    }
}

impl Stream for CarriageReturnLineFeedDelimitedStream {
    type Item=Box<Vec<u8>>;
    type Error=io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        while !self.build.ends_with(b"\r\n") {
            match try!(self.stream.poll()) {
                Async::Ready(Some(byte)) => self.build.push(byte),
                Async::NotReady | Async::Ready(None) => return Ok(Async::NotReady),
            }
        }
        let mut result = Vec::new();
        mem::swap(&mut result, &mut self.build);
        println!("{:?}", str::from_utf8(result.as_slice()).unwrap());
        Ok(Async::Ready(Some(Box::new(result))))
    }
}

