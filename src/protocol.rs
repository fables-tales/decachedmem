use std::io;
use tokio_proto::pipeline::ServerProto;

use tokio_core::io::{Io, Framed};
use codec::MemcachedCodec;
use memcached;

pub struct MemcachedProto {}

impl MemcachedProto {
    pub fn new() -> Self {
        MemcachedProto {}
    }
}

impl<T: Io + 'static> ServerProto<T> for MemcachedProto {
    type Request = memcached::Request;
    type Response = memcached::Reply;

    type Transport = Framed<T, MemcachedCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(MemcachedCodec::new()))
    }
}
