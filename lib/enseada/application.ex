defmodule Enseada.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  def start(_type, _args) do
    children = [
      {Plug.Cowboy, scheme: :http, plug: EnseadaWeb.Router, options: [port: 3000]}
    ]

    opts = [strategy: :one_for_one, name: Enseada.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
