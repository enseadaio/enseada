defmodule Enseada.MixProject do
  use Mix.Project

  def project do
    [
      app: :enseada,
      version: "0.1.0",
      elixir: "~> 1.9",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      aliases: aliases()
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger],
      mod: {Enseada.Application, []}
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:plug_cowboy, "~> 2.0"},
      {:jason, "~> 1.1"},
      {:corsica, "~> 1.1"},
      {:ex_doc, "~> 0.20", only: :dev, runtime: false}
    ]
  end

  defp aliases do
    [
      start: "run --no-halt"
    ]
  end
end
