use std::io;
use std::sync::{Arc, Mutex};
use tokio_service::Service;
use futures::{future, Future, BoxFuture};
use memcached;

pub struct MemcachedService {
    store: Arc<Mutex<memcached::Store>>,
}

impl MemcachedService {
    pub fn new(store: Arc<Mutex<memcached::Store>>) -> Self {
        MemcachedService { store: store }
    }

    fn process_get(&self, key: memcached::Key) -> <MemcachedService as Service>::Response {
        println!("here");
        let store = self.store.lock().unwrap();
        println!("here2");
        match store.get(&key) {
            Some(value) => memcached::Reply::Value(key, value),
            None => memcached::Reply::NotFound,
        }
    }

    fn process_set(&self,
                   key: memcached::Key,
                   body: Vec<u8>)
                   -> <MemcachedService as Service>::Response {
        let mut store = self.store.lock().unwrap();
        store.set(key, body);
        memcached::Reply::Stored
    }
}

impl Service for MemcachedService {
    // These types must match the corresponding protocol types:
    type Request = memcached::Request;
    type Response = memcached::Reply;

    // For non-streaming protocols, service errors are always io::Error
    type Error = io::Error;

    // The future for computing the response; box it for simplicity.
    type Future = BoxFuture<Self::Response, Self::Error>;

    // Produce a future for computing a response from a request.
    fn call(&self, req: Self::Request) -> Self::Future {
        let response = match req.command {
            memcached::Command::Get => self.process_get(req.key),
            memcached::Command::Set => self.process_set(req.key, req.body.unwrap()),
        };

        future::ok(response).boxed()
    }
}
