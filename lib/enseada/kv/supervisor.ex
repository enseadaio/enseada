defmodule Enseada.KV.Supervisor do
  use Supervisor

  @adapter Application.get_env(:enseada, :kv_adapter, Enseada.KV.InMemory)

  def start_link(init_arg) do
    Supervisor.start_link(__MODULE__, init_arg, name: __MODULE__)
  end

  @impl true
  def init(_) do
    children = [{adapter(), buckets: [:repos]}]

    Supervisor.init(children, strategy: :one_for_one, name: __MODULE__)
  end

  def adapter(), do: @adapter
end