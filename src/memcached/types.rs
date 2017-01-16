#[derive(PartialEq, Debug)]
pub struct Key(pub Vec<u8>);

#[derive(PartialEq, Debug)]
pub enum Command {
    Get,
    Set,
}

pub enum ReplyType {
    Error,
    Stored,
    Value,
}

#[derive(PartialEq, Debug)]
pub struct Request {
    command: Command,
    key: Key,
    body: Option<Vec<u8>>,
}

impl Request {
    pub fn new(command: Command, key: Key) -> Self {
        Request {
            command: command,
            key: key,
            body: None,
        }
    }

    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = Some(body);
    }
}

pub struct Reply {
    reply_type: ReplyType,
    key: Option<Key>,
    length: Option<u64>,
}

impl Reply {
    pub fn serialize(self) -> Vec<u8> {
        match self.reply_type {
            ReplyType::Error => b"ERROR"[..].to_vec(),
            ReplyType::Stored => b"STORED"[..].to_vec(),
            ReplyType::Value => {
                let mut build = Vec::new();
                build.extend_from_slice(b"VALUE ");
                build.extend_from_slice(self.key.unwrap().0.as_slice());
                build.extend_from_slice(b" 3 ");
                build.extend_from_slice(self.length.unwrap().to_string().as_bytes());
                build
            }
        }
    }
}
