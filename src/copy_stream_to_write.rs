use futures::stream::Stream;
use futures::{Poll, Async};
use std::io::{self, Write};

pub struct CopyStreamToWrite<S: Stream<Item = u8, Error = io::Error>, W: Write> {
    stream: S,
    write: W,
}

impl<S: Stream<Item = u8, Error = io::Error>, W: Write> CopyStreamToWrite<S, W> {
    pub fn new(stream: S, write: W) -> CopyStreamToWrite<S, W> {
        CopyStreamToWrite {
            stream: stream,
            write: write,
        }
    }
}

impl<S: Stream<Item = u8, Error = io::Error>, W: Write> Stream for CopyStreamToWrite<S, W> {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let poll = try_nb!(self.stream.poll());
        let result = match poll {
            Async::Ready(Some(x)) => {
                try_nb!(self.write.write(&[x]));
                Async::Ready(Some(()))
            }
            Async::Ready(None) => Async::Ready(None),
            Async::NotReady => Async::NotReady,
        };
        Ok(result)
    }
}
