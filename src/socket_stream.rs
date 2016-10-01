use std::io::{self, Read};
use futures::stream::Stream;
use tokio_core::io::{ReadHalf, Io};
use futures::Poll;
use futures::Async;

pub struct SocketStream<T: Io> {
    read: ReadHalf<T>,
}

impl<T> SocketStream<T>
    where T: Io
{
    pub fn new(read: ReadHalf<T>) -> SocketStream<T> {
        SocketStream { read: read }
    }
}


impl<T> Stream for SocketStream<T>
    where T: Io
{
    type Item = u8;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let ready = self.read.poll_read();
        match ready {
            Async::Ready(_) => {
                let mut buf = [0; 1];
                let read_result = try_nb!(self.read.read(&mut buf));
                match read_result {
                    0 => Ok(Async::Ready(None)),
                    _ => Ok(Async::Ready(Some(buf[0]))),
                }
            }
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}
