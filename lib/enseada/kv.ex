defmodule Enseada.KV do
  @adapter Enseada.KV.Supervisor.adapter()

  def get(bucket, key) do
    @adapter.get(bucket, key)
  end

  def find(bucket, match) do
    @adapter.find(bucket, match)
  end

  def put(bucket, key, value) do
    @adapter.put(bucket, key, value)
  end
end