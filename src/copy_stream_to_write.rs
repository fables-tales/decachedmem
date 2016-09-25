use futures::stream::Stream;
use futures::{Future, Poll, Async};
use std::io::{self, Write};
use std::str;

pub struct CopyStreamToWrite<S: Stream<Item=Vec<u8>, Error=io::Error>, W: Write> {
    stream: S,
    write: W,
}

impl <S: Stream<Item=Vec<u8>, Error=io::Error>, W: Write> CopyStreamToWrite<S, W> {
    pub fn new(stream: S, write: W) -> CopyStreamToWrite<S, W> {
        CopyStreamToWrite {
            stream: stream,
            write: write,
        }
    }
}

impl <S: Stream<Item=Vec<u8>, Error=io::Error>, W: Write> Stream for CopyStreamToWrite<S, W> {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let poll = try_nb!(self.stream.poll()) ;
        let result = match poll {
            Async::Ready(Some(mut x)) => {
                println!("{:?}", str::from_utf8(x.as_slice()).unwrap());
                println!("calling write");
                try_nb!(self.write.write(x.as_slice()));
                try_nb!(self.write.flush());
                Async::Ready(Some(()))
            },
            Async::Ready(None) => { Async::Ready(None) },
            Async::NotReady => { Async::NotReady }
        };
        Ok(result)
    }
}
