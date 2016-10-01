require "spec_helper"
require "memcached"

RSpec.describe "decachedmem" do
  let(:cache) { Memcached.new("localhost:11211") }
  let(:cache2) { Memcached.new("localhost:11211") }
  it "doesn't choke when setting a key" do
    expect {
      cache.set("foo", "37")
    }.not_to raise_error
  end

  it "doesn't choke when setting the key many times" do
    expect {
      cache.set("foo", "37")
      cache.set("foo", "37")
    }.not_to raise_error
  end

  it "can get a value it has set" do
    cache.set("foo", "37")
    expect(cache.get("foo")).to eq("37")
  end

  it "supports multiple connections" do
    cache.set("foo1", "37")
    cache2.set("foo2", "12")
    aggregate_failures do
      expect(cache.get("foo2")).to eq("12")
      expect(cache2.get("foo1")).to eq("37")
    end
  end
end
