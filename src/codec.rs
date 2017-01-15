use std::io;
use std::str;
use tokio_core::io::{Codec, EasyBuf};
use tokio_proto::streaming::pipeline::Frame;
use memcached;

type InType = Frame<memcached::Request, EasyBuf, io::Error>;

fn parse_bytes(expected_to_be_a_byte_count: &[u8]) -> Option<u64> {
    let byte_string = str::from_utf8(expected_to_be_a_byte_count).ok();
    let byte_count = byte_string.and_then(|byte_string| byte_string.parse().ok());
    byte_count
}

enum CodecState {
    AwaitingHeader,
    ReadingBody,
    Error,
}

pub struct MemcachedCodec {
    state: CodecState,
    bytes_to_read: Option<u64>,
}

impl MemcachedCodec {
    fn new() -> Self {
        MemcachedCodec {
            state: CodecState::AwaitingHeader,
            bytes_to_read: None,
        }
    }

    fn handle_error(&mut self) -> io::Result<Option<InType>> {
        self.state = CodecState::Error;
        Err(io::Error::new(io::ErrorKind::Other, "Codec is in error state"))
    }

    fn read_header(&mut self, buf: &mut EasyBuf) -> io::Result<Option<InType>> {
        if let Some(i) = buf.as_slice().windows(2).position(|b| b == b"\r\n") {
            let line = buf.drain_to(i + 1);

            // take the \r\n off
            buf.drain_to(1);

            let mut space_index = 0;
            let mut spaces_encountered = 0;
            let mut command = None;
            let mut key = None;
            let mut bytes_to_read = None;
            for cursor in 0..i + 1 {
                let part = &line.as_slice()[space_index..cursor];

                let next_to_consider = line.as_slice()[cursor];
                if next_to_consider == b' ' || next_to_consider == b'\r' {
                    if spaces_encountered == 0 {
                        match part {
                            b"GET" => {
                                command = Some(memcached::Command::Get);
                            }
                            b"SET" => {
                                command = Some(memcached::Command::Set);
                            }
                            _ => {}
                        }

                    } else if spaces_encountered == 1 {
                        key = Some(memcached::Key(part.to_vec()));
                    } else if spaces_encountered == 4 {
                        bytes_to_read = parse_bytes(part);
                    }

                    spaces_encountered += 1;
                    space_index = cursor + 1;
                }
            }

            match (command, key) {
                // TODO: pass through spaces encountered here to validate it's the right number
                // for the passed command
                (Some(command), Some(key)) => self.consume_header(command, key, bytes_to_read),
                _ => self.handle_error(),
            }
        } else {
            Ok(None)
        }
    }

    fn consume_header(&mut self,
                      command: memcached::Command,
                      key: memcached::Key,
                      bytes_to_read: Option<u64>)
                      -> io::Result<Option<InType>> {
        match command {
            memcached::Command::Get => {
                Ok(Some(Frame::Message {
                    message: memcached::Request::new(memcached::Command::Get, key),
                    body: false,
                }))
            }
            memcached::Command::Set => {
                match bytes_to_read {
                    Some(count) => {
                        self.state = CodecState::ReadingBody;
                        self.bytes_to_read = Some(count);

                        Ok(Some(Frame::Message {
                            message: memcached::Request::new(memcached::Command::Set, key),
                            body: true,
                        }))
                    }
                    None => {
                        Err(io::Error::new(io::ErrorKind::Other,
                                           "didn't get a valid byte count with a set request"))
                    }
                }
            }
        }
    }

    fn read_body(&mut self, buf: &mut EasyBuf) -> io::Result<Option<InType>> {
        match self.bytes_to_read {
            Some(count) => {
                if buf.len() >= count as usize {
                    let body = buf.drain_to(count as usize);
                    buf.drain_to(2);
                    self.state = CodecState::AwaitingHeader;
                    Ok(Some(Frame::Body { chunk: Some(body) }))
                } else {
                    Ok(None)
                }
            }
            None => {
                panic!("Literally it should not be possible to get to read_body with a None \
                        bytes_to_read")
            }
        }
    }
}

impl Codec for MemcachedCodec {
    type In = InType;
    type Out = Frame<memcached::Reply, Vec<u8>, io::Error>;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        match self.state {
            CodecState::AwaitingHeader => self.read_header(buf),
            CodecState::ReadingBody => self.read_body(buf),
            CodecState::Error => self.handle_error(),
        }
    }

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> io::Result<()> {

        match msg {
            Frame::Message { message, body } => {
                buf.extend_from_slice(&message.serialize());
                buf.extend_from_slice(b"\r\n");
                Ok(())
            }
            Frame::Body { chunk } => {
                if let Some(chunk) = chunk {
                    buf.extend_from_slice(chunk.as_slice());
                    buf.extend_from_slice(b"\r\n");
                }
                Ok(())
            }
            Frame::Error { error } => {
                // TODO: we could use this for memcached errors
                // if we were 2 cool for skool
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio_proto::streaming::pipeline::Frame;
    use tokio_core::io::{Codec, EasyBuf};
    use super::MemcachedCodec;
    use memcached;

    #[test]
    fn decode_get_works() {
        let mut codec = MemcachedCodec::new();
        let mut buf = EasyBuf::from("GET foo\r\n".as_bytes().to_vec());

        let result = codec.decode(&mut buf);

        let frame = result.unwrap().unwrap();

        match frame {
            Frame::Message { message, body } => {
                assert_eq!(message,
                           memcached::Request::new(memcached::Command::Get,
                                                   memcached::Key("foo".as_bytes().to_vec())))
            }
            _ => {
                panic!("didn't get a message back from a get");
            }
        }

    }

    #[test]
    fn decode_get_with_trailing_space_works() {
        let mut codec = MemcachedCodec::new();
        let mut buf = EasyBuf::from("GET foo \r\n".as_bytes().to_vec());

        let result = codec.decode(&mut buf);

        let frame = result.unwrap().unwrap();

        match frame {
            Frame::Message { message, body } => {
                assert_eq!(message,
                           memcached::Request::new(memcached::Command::Get,
                                                   memcached::Key("foo".as_bytes().to_vec())))
            }
            _ => {
                panic!("didn't get a message back from a get");
            }
        }

    }

    #[test]
    fn multiple_decode_works() {
        let mut codec = MemcachedCodec::new();
        let mut buf = EasyBuf::from("GET foo\r\n".as_bytes().to_vec());

        let result = codec.decode(&mut buf);
        let frame = result.unwrap().unwrap();

        match frame {
            Frame::Message { message, body } => {
                assert_eq!(message,
                           memcached::Request::new(memcached::Command::Get,
                                                   memcached::Key("foo".as_bytes().to_vec())))
            }
            _ => {
                panic!("didn't get a message back from a get");
            }
        }

        let mut buf = EasyBuf::from("GET foo2\r\n".as_bytes().to_vec());

        let result = codec.decode(&mut buf);
        let frame = result.unwrap().unwrap();

        match frame {
            Frame::Message { message, body } => {
                assert_eq!(message,
                           memcached::Request::new(memcached::Command::Get,
                                                   memcached::Key("foo2".as_bytes().to_vec())))
            }
            _ => {
                panic!("didn't get a message back from a get");
            }
        }
    }

    #[test]
    fn set_works() {
        let mut codec = MemcachedCodec::new();
        let mut buf = EasyBuf::from("SET foo flag 0 4\r\nabcd\r\n".as_bytes().to_vec());

        let result = codec.decode(&mut buf);
        let frame = result.unwrap().unwrap();

        match frame {
            Frame::Message { message, body } => {
                assert_eq!(message,
                           memcached::Request::new(memcached::Command::Set,
                                                   memcached::Key("foo".as_bytes().to_vec())))
            }
            _ => {
                panic!("didn't get a message back from a get");
            }
        }

        let result = codec.decode(&mut buf);
        let frame = result.unwrap().unwrap();

        match frame {
            Frame::Body { chunk } => assert_eq!(chunk.unwrap().as_slice(), b"abcd"),
            _ => {
                panic!("didn't get a message back from a get");
            }
        }
    }

    #[test]
    fn set_then_get_works() {
        let mut codec = MemcachedCodec::new();
        let mut buf = EasyBuf::from("SET foo flag 0 4\r\nabcd\r\nGET foo\r\n".as_bytes().to_vec());

        let result = codec.decode(&mut buf);
        let frame = result.unwrap().unwrap();

        match frame {
            Frame::Message { message, body } => {
                assert_eq!(message,
                           memcached::Request::new(memcached::Command::Set,
                                                   memcached::Key("foo".as_bytes().to_vec())))
            }
            _ => {
                panic!("didn't get a message back from a get");
            }
        }

        let result = codec.decode(&mut buf);
        let frame = result.unwrap().unwrap();

        match frame {
            Frame::Body { chunk } => assert_eq!(chunk.unwrap().as_slice(), b"abcd"),
            _ => {
                panic!("didn't get a message back from a get");
            }
        }

        let result = codec.decode(&mut buf);
        let frame = result.unwrap().unwrap();

        match frame {
            Frame::Message { message, body } => {
                assert_eq!(message,
                           memcached::Request::new(memcached::Command::Get,
                                                   memcached::Key("foo".as_bytes().to_vec())))
            }
            _ => {
                panic!("didn't get a message back from a get");
            }
        }
    }
}
