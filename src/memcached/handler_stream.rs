use futures::stream::Stream;
use futures::{Poll, Async};
use memcached::stream::MemcachedProtcolStream;
use memcached::store::Store;
use memcached::types::*;
use std::io;
use std::cell::RefCell;
use std::rc::Rc;

pub struct MemcachedHandlerStream<T: Stream<Item = Vec<u8>, Error = io::Error>> {
    protocol_stream: MemcachedProtcolStream<T>,
    store: Rc<RefCell<Store>>,
}

impl<T: Stream<Item = Vec<u8>, Error = io::Error>> MemcachedHandlerStream<T> {
    pub fn new(store: Rc<RefCell<Store>>,
               protocol_stream: MemcachedProtcolStream<T>)
               -> MemcachedHandlerStream<T> {
        MemcachedHandlerStream {
            store: store,
            protocol_stream: protocol_stream,
        }
    }
}

impl<T: Stream<Item = Vec<u8>, Error = io::Error>> Stream for MemcachedHandlerStream<T> {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let poll = try!(self.protocol_stream.poll());
        let result = match poll {
            Async::Ready(Some(message)) => {
                let mut store = self.store.borrow_mut();
                Async::Ready(Some(handle_message(&mut store, message)))
            }
            Async::Ready(None) => Async::Ready(None),
            Async::NotReady => Async::NotReady,
        };

        println!("handler {:?}", result);
        Ok(result)
    }
}

fn handle_message(store: &mut Store, message: MemcachedFrame) -> Vec<u8> {
    let key = message.header.key;
    let command = message.header.command_name;
    let bytes = message.bytes;
    let response = match command {
        MemcachedCommandName::Get => handle_get(&key, store),
        MemcachedCommandName::Set => handle_set(&key, bytes, store),
    };

    response
}


fn handle_get(key: &MemcachedKey, store: &mut Store) -> Vec<u8> {
    let value = store.get(key);
    match value {
        Some(x) => found_response(key, x),
        None => not_found_response(),
    }
}

fn handle_set(key: &MemcachedKey, bytes: MemcachedValue, store: &mut Store) -> Vec<u8> {
    store.set(key, bytes);

    b"STORED\r\n".to_vec()
}

fn found_response(key: &MemcachedKey, value: &MemcachedValue) -> Vec<u8> {
    let mut build = vec![];
    // header
    build.extend_from_slice(b"VALUE ");

    // key
    build.extend_from_slice(&key);
    build.extend_from_slice(b" ");

    // flags
    build.extend_from_slice(b"0 ");

    // value length
    build.extend_from_slice(value.len().to_string().as_bytes());
    build.extend_from_slice(b" ");

    // end of header
    build.extend_from_slice(b"\r\n");

    // value
    build.extend_from_slice(&value);
    build.extend_from_slice(b"\r\n");

    // end frame
    build.extend_from_slice(b"END\r\n");
    build
}

fn not_found_response() -> Vec<u8> {
    let mut build = vec![];
    build.extend_from_slice(b"END\r\n");
    build
}
