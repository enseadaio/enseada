defmodule Enseada.Database.Initialzer do
  use Task, restart: :transient
  require Logger

  def start_link(arg) do
    Task.start_link(__MODULE__, :run, [arg])
  end

  def run(arg) do
    with {:ok, _} <- repositories(),
         {:ok, _} <- users() do
      Logger.info("Database initialization completed")
    else
      {:error, error} -> Logger.error("Database initialization failed: #{inspect(error)}")
    end
  end

  defp repositories() do
    Enseada.Database.database_exists?("repositories")
    |> repositories()
  end

  defp repositories(false) do
    case Enseada.Database.create_database("repositories") do
      {:ok, db} ->
        Logger.info("Created database repositories")
        {:ok, db}

      {:error, error} ->
        {:error, {"repositories", error}}
    end
  end

  defp repositories(true) do
    Enseada.Database.get_database("repositories")
  end

  defp users() do
    Enseada.Database.database_exists?("users")
    |> users()
  end

  defp users(false) do
    case Enseada.Database.create_database("users") do
      {:ok, db} ->
        Logger.info("Created database users")
        {:ok, db}

      {:error, error} ->
        {:error, {"users", error}}
    end
  end

  defp users(true) do
    Enseada.Database.get_database("users")
  end
end
