pub type MemcachedKey = Vec<u8>;
pub type MemcachedValue = Vec<u8>;
pub type MemcachedFlags = u16;
pub type MemcachedExpTime = i32;

#[derive(Debug)]
pub enum MemcachedCommandName {
    Get,
    Set
}

#[derive(Debug)]
pub struct MemcachedFrameHeader {
    pub command_name: MemcachedCommandName,
    pub key: MemcachedKey,
    pub flags: MemcachedFlags,
    pub exptime: MemcachedExpTime,
    pub byte_count: usize,
}

impl MemcachedFrameHeader {
    pub fn as_dataless_frame(self) -> MemcachedFrame {
        MemcachedFrame {
            header: self,
            bytes: vec!(),
        }
    }
}

#[derive(Debug)]
pub struct MemcachedFrame {
    pub header: MemcachedFrameHeader,
    pub bytes: MemcachedValue,
}

impl MemcachedFrame {
    pub fn new(header: MemcachedFrameHeader, bytes: MemcachedValue) -> MemcachedFrame {
        MemcachedFrame {
            header: header,
            bytes: bytes,
        }
    }
}
