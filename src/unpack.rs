use futures::stream::Stream;
use futures::{Future, Poll, Async};

pub struct Unpack<I, T: Stream<Item = Vec<I>>> {
    buffer: Vec<I>,
    stream: T,
}

impl<I, T: Stream<Item = Vec<I>>> Unpack<I, T> {
    pub fn new(stream: T) -> Self {
        Unpack {
            buffer: vec![],
            stream: stream,
        }
    }
}

impl<I, T: Stream<Item = Vec<I>>> Stream for Unpack<I, T> {
    type Item = I;
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if self.buffer.len() == 0 {
            let poll = try!(self.stream.poll());
            match poll {
                Async::Ready(Some(mut x)) => {
                    self.buffer.append(&mut x);
                }
                Async::Ready(None) => return Ok(Async::Ready(None)),
                Async::NotReady => return Ok(Async::NotReady),
            }
        }

        let next_item = self.buffer.remove(0);
        Ok(Async::Ready(Some(next_item)))
    }
}
