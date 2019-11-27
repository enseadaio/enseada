defmodule Enseada.KV.InMemoryTest do
  use ExUnit.Case, async: true

  alias Enseada.KV.InMemory

  @moduletag :capture_log

  doctest InMemory

  setup do
    server = start_supervised!(InMemory)
    %{server: server}
  end

  test "stores a value" do
    assert :ok = InMemory.put(:users, "123-456", %{name: "Test"})
    assert {:ok, %{name: "Test"}} = InMemory.get(:users, "123-456")
  end
end
