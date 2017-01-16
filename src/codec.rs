use std::io;
use std::mem;
use std::str;
use tokio_core::io::{Codec, EasyBuf};
use memcached;

fn parse_bytes(expected_to_be_a_byte_count: &[u8]) -> Option<u64> {
    let byte_string = str::from_utf8(expected_to_be_a_byte_count).ok();
    let byte_count = byte_string.and_then(|byte_string| byte_string.parse().ok());
    byte_count
}

#[derive(PartialEq)]
enum CodecState {
    AwaitingHeader,
    ReadingBody,
    Error,
}

pub struct MemcachedCodec {
    state: CodecState,
    partial: Option<memcached::Request>,
    bytes_to_read: Option<u64>,
}

impl MemcachedCodec {
    pub fn new() -> Self {
        MemcachedCodec {
            state: CodecState::AwaitingHeader,
            bytes_to_read: None,
            partial: None,
        }
    }

    fn handle_error(&mut self) -> io::Result<Option<memcached::Request>> {
        self.state = CodecState::Error;
        Err(io::Error::new(io::ErrorKind::Other, "Codec is in error state"))
    }

    fn read_header(&mut self, buf: &mut EasyBuf) -> io::Result<Option<memcached::Request>> {
        if let Some(i) = buf.as_slice().windows(2).position(|b| b == b"\r\n") {
            let line = buf.drain_to(i + 1);

            // take the \r\n off
            buf.drain_to(1);

            println!("{:?}", line.as_slice());

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
                            b"get" => {
                                command = Some(memcached::Command::Get);
                            }
                            b"set" => {
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

            println!("{:?} {:?}", command, key);
            match (command, key) {
                // TODO: pass through spaces encountered here to validate it's the right number
                // for the passed command
                (Some(command), Some(key)) => {
                    let mut res = self.consume_header(command, key, bytes_to_read);
                    if self.state == CodecState::ReadingBody {
                        res = self.decode(buf);
                    }

                    res
                }
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
                      -> io::Result<Option<memcached::Request>> {
        match command {
            memcached::Command::Get => {
                println!("have get");
                Ok(Some(memcached::Request::new(memcached::Command::Get, key)))
            }
            memcached::Command::Set => {
                match bytes_to_read {
                    Some(count) => {
                        self.state = CodecState::ReadingBody;
                        self.bytes_to_read = Some(count);
                        self.partial = Some(memcached::Request::new(memcached::Command::Set, key));

                        Ok(None)
                    }
                    None => {
                        Err(io::Error::new(io::ErrorKind::Other,
                                           "didn't get a valid byte count with a set request"))
                    }
                }
            }
        }
    }

    fn read_body(&mut self, buf: &mut EasyBuf) -> io::Result<Option<memcached::Request>> {
        match self.bytes_to_read {
            Some(count) => {
                if buf.len() >= count as usize {
                    let body = buf.drain_to(count as usize);
                    buf.drain_to(2);
                    self.state = CodecState::AwaitingHeader;
                    let mut partial = None;
                    mem::swap(&mut partial, &mut self.partial);
                    let mut partial = partial.unwrap();
                    partial.set_body(body.as_slice().to_vec());
                    self.bytes_to_read = None;
                    Ok(Some(partial))
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
    type In = memcached::Request;
    type Out = memcached::Reply;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        println!("-------------------");
        println!("buf before {:?}", buf.as_slice());

        let res = match self.state {
            CodecState::AwaitingHeader => self.read_header(buf),
            CodecState::ReadingBody => self.read_body(buf),
            CodecState::Error => self.handle_error(),
        };
        println!("buf after {:?}", buf.as_slice());
        println!("-------------------");
        res
    }

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.extend_from_slice(&msg.serialize());
        buf.extend_from_slice(b"\r\n");
        println!("sending: {:?}", buf.as_slice());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokio_core::io::{Codec, EasyBuf};
    use super::MemcachedCodec;
    use memcached;

    #[test]
    fn decode_get_works() {
        let mut codec = MemcachedCodec::new();
        let mut buf = EasyBuf::from("get foo\r\n".as_bytes().to_vec());

        let result = codec.decode(&mut buf);

        let frame = result.unwrap().unwrap();
        assert_eq!(frame,
                   memcached::Request::new(memcached::Command::Get,
                                           memcached::Key("foo".as_bytes().to_vec())));

    }

    #[test]
    fn decode_get_with_trailing_space_works() {
        let mut codec = MemcachedCodec::new();
        let mut buf = EasyBuf::from("get foo \r\n".as_bytes().to_vec());

        let result = codec.decode(&mut buf);

        let frame = result.unwrap().unwrap();
        assert_eq!(frame,
                   memcached::Request::new(memcached::Command::Get,
                                           memcached::Key("foo".as_bytes().to_vec())))

    }

    #[test]
    fn multiple_decode_works() {
        let mut codec = MemcachedCodec::new();
        let mut buf = EasyBuf::from("get foo\r\n".as_bytes().to_vec());

        let result = codec.decode(&mut buf);
        let frame = result.unwrap().unwrap();
        assert_eq!(frame,
                   memcached::Request::new(memcached::Command::Get,
                                           memcached::Key("foo".as_bytes().to_vec())));

        let mut buf = EasyBuf::from("get foo2\r\n".as_bytes().to_vec());

        let result = codec.decode(&mut buf);
        let frame = result.unwrap().unwrap();

        assert_eq!(frame,
                   memcached::Request::new(memcached::Command::Get,
                                           memcached::Key("foo2".as_bytes().to_vec())));
    }

    #[test]
    fn set_works() {
        let mut codec = MemcachedCodec::new();
        let mut buf = EasyBuf::from("set foo flag 0 4\r\nabcd\r\n".as_bytes().to_vec());

        let result = codec.decode(&mut buf);
        let frame = result.unwrap().unwrap();

        let mut expected = memcached::Request::new(memcached::Command::Set,
                                                   memcached::Key("foo".as_bytes().to_vec()));
        expected.set_body(b"abcd"[..].to_vec());
        assert_eq!(frame, expected);
    }

    #[test]
    fn set_then_get_works() {
        let mut codec = MemcachedCodec::new();
        let mut buf = EasyBuf::from("set foo flag 0 4\r\nabcd\r\nget foo\r\n".as_bytes().to_vec());

        let result = codec.decode(&mut buf);
        let frame = result.unwrap().unwrap();
        let mut expected = memcached::Request::new(memcached::Command::Set,
                                                   memcached::Key("foo".as_bytes().to_vec()));

        expected.set_body(b"abcd"[..].to_vec());

        assert_eq!(frame, expected);

        let result = codec.decode(&mut buf);
        let frame = result.unwrap().unwrap();
        assert_eq!(frame,
                   memcached::Request::new(memcached::Command::Get,
                                           memcached::Key("foo".as_bytes().to_vec())));
    }
}
