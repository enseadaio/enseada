defmodule Enseada.KV.Adapter do
  @callback get(bucket :: atom(), key :: atom()) :: {:ok, any()} | {:error, String.t()}
  @callback put(bucket :: atom(), key :: atom(), value :: any()) :: :ok | {:error, String.t()}
end
