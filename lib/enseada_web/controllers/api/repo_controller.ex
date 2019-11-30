defmodule EnseadaWeb.Api.RepoController do
  use EnseadaWeb, :controller

  alias Enseada.Mapper
  alias Enseada.Repositories
  alias Enseada.Maven.Templates
  alias Enseada.Maven.Storage

  def index(conn, %{"type" => type}) do
    {:ok, repos} = Repositories.list()
    json(conn, repos)
  end

  def index(conn, _params), do: send_resp(conn, 400, "")

  def create(conn, params = %{"type" => type}) do
    case create_repo(type, params) do
      {:ok, repo} ->
        body = Templates.metadata(repo.group_id, repo.artifact_id)
        Storage.store({%{filename: "maven-metadata.xml", binary: body}, scope(params)})

        conn
        |> put_status(201)
        |> json(repo)

      {:error, reason} ->
        conn
        |> put_status(400)
        |> json(%{message: reason})
    end
  end

  def create(conn, _params), do: send_resp(conn, 400, "")

  defp create_repo("maven", %{"group_id" => group_id, "artifact_id" => artifact_id}) do
    repo = %{
      type: "maven",
      name: "#{group_id}:#{artifact_id}",
      group_id: group_id,
      artifact_id: artifact_id
    }

    Repositories.create(repo)
  end

  defp create_repo(_, _), do: {:error, "unknown repo type"}

  defp scope(%{"group_id" => group_id, "artifact_id" => artifact_id}) do
    prefix = String.replace(group_id, ".", "/")
    "#{prefix}/#{artifact_id}"
  end
end
