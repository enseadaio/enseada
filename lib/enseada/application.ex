defmodule Enseada.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  def start(_type, _args) do
    children = [
      EnseadaWeb.Endpoint,
      Enseada.Database,
      Enseada.Database.Initialzer
    ]

    opts = [strategy: :one_for_one, name: Enseada.Supervisor]
    Supervisor.start_link(children, opts)
  end

  # Tell Phoenix to update the endpoint configuration
  # whenever the application is updated.
  def config_change(changed, _new, removed) do
    EnseadaWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end
