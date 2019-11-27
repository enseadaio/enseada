defmodule Enseada.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  require Logger

  def start(_type, _args) do
    port = Application.get_env(:enseada, :port, 3000)

    children = [
      {Plug.Cowboy, scheme: :http, plug: EnseadaWeb.Router, options: [port: port]}
    ]

    opts = [strategy: :one_for_one, name: Enseada.Supervisor]
    Logger.info("Started Enseada on port #{port}")
    Supervisor.start_link(children, opts)
  end
end
