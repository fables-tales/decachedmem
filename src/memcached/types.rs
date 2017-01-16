#[derive(PartialEq, Eq, Debug, Hash)]
pub struct Key(pub Vec<u8>);

#[derive(PartialEq, Debug)]
pub enum Command {
    Get,
    Set,
}

#[derive(PartialEq, Debug)]
pub struct Request {
    pub command: Command,
    pub key: Key,
    pub body: Option<Vec<u8>>,
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

pub enum Reply {
    Stored,
    Value(Key, Vec<u8>),
    NotFound,
}

impl Reply {
    pub fn serialize(self) -> Vec<u8> {
        match self {
            Reply::Stored => b"STORED".to_vec(),
            Reply::NotFound => b"NOT_FOUND".to_vec(),
            Reply::Value(key, body) => {
                let mut build = Vec::new();
                build.extend_from_slice(b"VALUE ");
                build.extend_from_slice(key.0.as_slice());
                build.extend_from_slice(b" 3 ");
                build.extend_from_slice(body.len().to_string().as_bytes());
                build.extend_from_slice(b"\r\n");
                build.extend_from_slice(body.as_slice());
                build
            }
        }
    }
}
