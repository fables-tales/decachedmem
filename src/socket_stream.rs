use std::io::{self, Read};
use futures::stream::Stream;
use futures::{Poll, Async};
use tokio_core::TcpStream;
pub struct SocketStream {
    socket: TcpStream,
}

impl SocketStream {
    pub fn new(socket: TcpStream) -> SocketStream {
        SocketStream {
            socket: socket
        }
    }
}


impl Stream for SocketStream {
    type Item=u8;
    type Error=io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let ready = self.socket.poll_read();
        match ready {
            Ok(_) => {
                let mut buf = [0; 1];
                let read_result = self.socket.read(&mut buf);
                match read_result {
                    Ok(0) => Ok(Async::Ready(None)),
                    Ok(_) => Ok(Async::Ready(Some(buf[0]))),
                    Err(e) => Err(e),
                }
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

