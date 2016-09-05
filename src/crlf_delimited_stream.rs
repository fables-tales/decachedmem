use std::io;
use futures::stream::Stream;
use futures::{Poll, Async};

fn last_two<T>(v: &Vec<T>) -> Option<Vec<T>> where T: Copy {
    if v.len() >= 2 {
        Some(vec!(v[v.len()-2], v[v.len()-1]))
    } else {
        None
    }
}

fn ends_with_carriage_return_line_feed(v: &Vec<u8>) -> bool {
    v[v.len()-2] == b'\r' && v[v.len()-1] == b'\n'
}


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
        loop {
            let poll = self.stream.poll();
            match poll {
                Ok(Async::Ready(Some(x))) => {
                    println!("have value: {}", x);
                    self.build.push(x);
                    match last_two(&self.build) {
                        Some(_) => {
                            if ends_with_carriage_return_line_feed(&self.build) {
                                let b = Box::new(self.build.to_vec());
                                self.build.clear();
                                return Ok(Async::Ready(Some(b)))
                            }
                        },
                        None => {},
                    }

                },
                Ok(Async::Ready(None)) => {
                    println!("have no value");
                    return Ok(Async::NotReady)
                }
                Ok(Async::NotReady) => {
                    println!("have not ready");
                    return Ok(Async::NotReady)
                },
                Err(e) => {
                    println!("e: {:?}", e);
                    return Err(e)
                }
            }
        }
    }
}

