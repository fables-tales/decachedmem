require "spec_helper"
require "memcached"

RSpec.describe "decachedmem" do
  let(:cache) { Memcached.new("localhost:8080") }
  it "doesn't choke when setting a key" do
    expect {
      cache.set("foo", "37")
    }.not_to raise_error
  end

  it "doesn't choke when setting the key many times" do
    expect {
      (100..300).to_a.sample.times do
        cache.set("foo", "37")
      end
    }.not_to raise_error
  end
end
