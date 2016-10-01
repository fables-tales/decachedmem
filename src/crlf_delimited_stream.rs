use std::{io, mem};
use futures::stream::Stream;
use futures::{Poll, Async};


pub struct CarriageReturnLineFeedDelimitedStream<T: Stream<Item = u8, Error = io::Error>> {
    stream: T,
    build: Vec<u8>,
}

impl<T: Stream<Item = u8, Error = io::Error>> CarriageReturnLineFeedDelimitedStream<T> {
    pub fn new(stream: T) -> CarriageReturnLineFeedDelimitedStream<T> {
        CarriageReturnLineFeedDelimitedStream {
            stream: stream,
            build: vec![],
        }
    }
}

impl<T: Stream<Item = u8, Error = io::Error>> Stream for CarriageReturnLineFeedDelimitedStream<T> {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        while !self.build.ends_with(b"\r\n") {
            match try!(self.stream.poll()) {
                Async::Ready(Some(byte)) => self.build.push(byte),
                Async::NotReady |
                Async::Ready(None) => return Ok(Async::NotReady),
            }
        }
        let mut complete_frame = Vec::new();
        mem::swap(&mut complete_frame, &mut self.build);
        Ok(Async::Ready(Some(complete_frame)))
    }
}
