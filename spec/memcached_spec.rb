require "spec_helper"
require "memcached"

RSpec.describe "decachedmem" do
  let(:cache) { Memcached.new("localhost:11211") }
  it "doesn't choke when setting a key" do
    expect {
      cache.set("foo", "37")
    }.not_to raise_error
  end

  it "doesn't choke when setting the key many times" do
    expect {
      p "iter start"
      cache.set("foo", "37")
      p "iter end"
      cache.set("foo", "37")
    }.not_to raise_error
  end
end
