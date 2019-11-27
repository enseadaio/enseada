defmodule Enseada.KV.InMemory do
  use GenServer
  @behaviour Enseada.KV.Adapter

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  @impl true
  def init(opts) do
    :users = :ets.new(:users, [:set, :protected, :named_table])
    {:ok, opts}
  end

  @impl true
  def get(bucket, key) do
    GenServer.call(__MODULE__, {:get, {bucket, key}})
  end

  @impl true
  def put(bucket, key, value) do
    GenServer.call(__MODULE__, {:put, {bucket, key, value}})
  end

  @impl true
  def handle_call({:get, {bucket, key}}, _, state) do
    res =
      case :ets.lookup(bucket, key) do
        [{^key, value}] -> {:ok, value}
        [] -> {:ok, nil}
      end

    {:reply, res, state}
  end

  @impl true
  def handle_call({:put, {bucket, key, value}}, _, state) do
    :ets.insert(bucket, {key, value})
    {:reply, :ok, state}
  end
end
