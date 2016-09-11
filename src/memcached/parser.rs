use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::mem;

use memcached::types::*;

#[derive(Debug)]
pub struct MemcachedParseError {
    description: String
}

impl MemcachedParseError {
    fn new(description: String) -> MemcachedParseError {
        MemcachedParseError {
            description: description
        }
    }

    fn from_err<T>(e: T) -> MemcachedParseError where T: Error + Sized {
        Self::new(e.description().into())
    }
}

impl Display for MemcachedParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "MemcachedError({})", self.description)
    }
}

impl Error for MemcachedParseError {
    fn description(&self) -> &str {
        &self.description
    }
}

fn parse_header_part<T>(part: &[u8]) -> Result<T, MemcachedParseError> where T: FromStr, T::Err: Error + Sized {
    let string = try!(String::from_utf8(part.to_vec()).map_err(|e| MemcachedParseError::from_err(e)));
    let parsed = try!(string.parse().map_err(|e| MemcachedParseError::from_err(e)));
    Ok(parsed)
}

fn parse_memcached_header(head: &Vec<u8>) -> Result<MemcachedFrameHeader, MemcachedParseError> {
    let parts: Vec<&[u8]> = head.split(|&byte| byte == b' ').collect();
    if parts.len() == 5 {
        println!("{:?} {:?}", parts[0], b"set");
        println!("{:?} {:?}", parts[0], b"get");
        let command = match parts[0] {
            b"set" => MemcachedCommandName::Set,
            b"get" => MemcachedCommandName::Get,
            _ => return Err(MemcachedParseError::new("invalid memcached command sent".into())),
        };
        let key = parts[1].to_vec();
        let flags = try!(parse_header_part(parts[2]));
        let exptime = try!(parse_header_part(parts[3]));
        println!("{:?}", parts[4]);
        let byte_count = try!(parse_header_part(&parts[4][0..parts[4].len()-2]));

        Ok(MemcachedFrameHeader {
            command_name: command,
            key: key,
            flags: flags,
            exptime: exptime,
            byte_count: byte_count,
        })
    } else {
        Err(MemcachedParseError{ description: "invalid memcached frame passed to header".into()})
    }
}

enum MemcachedParseState {
    NewHeader,
    AccumulatingSet,
    ErrorState,
}

pub struct MemcachedParseStateMachine {
    state: MemcachedParseState,
    buffer: Vec<u8>,
    partial_header: Option<MemcachedFrameHeader>,
}

impl MemcachedParseStateMachine {
    pub fn new() -> MemcachedParseStateMachine {
        MemcachedParseStateMachine {
            state: MemcachedParseState::NewHeader,
            buffer: vec!(),
            partial_header: None,
        }
    }

    // does not deal with splitting by CRLF, only pass in byte sequences that are already CRLF
    // delimited
    pub fn add_bytes(&mut self, bytes: &Vec<u8>) -> Result<Option<MemcachedFrame>, MemcachedParseError> {
        match self.state {
            MemcachedParseState::ErrorState => Err(MemcachedParseError::new("In error state from previous call".into())),
            MemcachedParseState::NewHeader => self.parse_header(bytes),
            MemcachedParseState::AccumulatingSet => self.accumulate(bytes),
        }
    }

    fn parse_header(&mut self, bytes: &Vec<u8>) -> Result<Option<MemcachedFrame>, MemcachedParseError> {
        let header = parse_memcached_header(bytes);
        match header {
            Ok(header) => Ok(self.transition_for(header)),
            Err(e) => {
                self.state = MemcachedParseState::ErrorState;
                Err(MemcachedParseError::from_err(e))
            }
        }
    }

    fn accumulate(&mut self, bytes: &Vec<u8>) -> Result<Option<MemcachedFrame>, MemcachedParseError> {
        self.buffer.extend_from_slice(bytes.as_slice());

        if self.buffer.len() >= self.partial_header.as_mut().unwrap().byte_count {
            self.buffer.truncate(self.partial_header.as_mut().unwrap().byte_count);

            let frame = self.produce_frame_and_reset();

            Ok(Some(frame))
        } else {
            Ok(None)
        }
    }

    fn produce_frame_and_reset(&mut self) -> MemcachedFrame {
        let mut new_buffer = vec!();
        mem::swap(&mut new_buffer, &mut self.buffer);

        let mut new_header = None;
        mem::swap(&mut new_header, &mut self.partial_header);

        self.state = MemcachedParseState::NewHeader;

        MemcachedFrame::new(new_header.unwrap(), new_buffer)
    }

    fn transition_for(&mut self, header: MemcachedFrameHeader) -> Option<MemcachedFrame> {
        match header.command_name {
            MemcachedCommandName::Get => Some(header.as_dataless_frame()),
            MemcachedCommandName::Set => {
                self.state = MemcachedParseState::AccumulatingSet;
                self.partial_header = Some(header);
                None
            }
        }
    }
}

