# Decachedmem

An (bad, partial) implementation of memcached in Rust using
[futures](https://github.com/alexcrichton/futures-rs) and
[tokio](https://github.com/tokio-rs/tokio-core).

At the moment it only supports the memcached `get` and `set` commands. To
demonstrate that those work, you'll see a set of RSpec test files attached which
manipulate the server.

## Making it go

1. You need both Ruby and Rust development tools on your local machine
2. Clone this repo
3. Run `cargo build && bundle install`
4. In one terminal run `cargo run`
5. In another terminal run `bundle exec rspec`

When you run `cargo run` you're starting the memcached server bound to port
`11211` (so you might get a conflict if you're running the real version of
memcached in the background). Hypothetically, you can attach any basic memcached
client to this server. As a note, the Ruby `dalli` gem doesn't work, because it
actually asks for stats from the server when it connects, which this one isn't
yet set up to do.
