defmodule EnseadaWeb.Api.RepoController do
  use EnseadaWeb, :controller

  alias Enseada.KV
  alias Enseada.Maven.Templates
  alias Enseada.Maven.Storage

  def index(conn, %{"type" => type}) do
    repos = KV.find(:repos, {:"$1", :"$2"})
            |> Enum.map(fn {_, repo} -> repo end)
    json(conn, repos)
  end
  def index(conn, _params), do: send_resp(conn, 400, "")

  def create(conn, %{"type" => type} = params) do
    case create_repo(type, params) do
      :ok ->
        body = Templates.metadata(params["group_id"], params["artifact_id"])
        Storage.store({%{filename: "maven-metadata.xml", binary: body}, scope(params)})
        send_resp(conn, 201, "")
      {:error, reason} ->
        conn
        |> put_status(400)
        |> json(%{message: reason})
    end
  end
  def create(conn, _params), do: send_resp(conn, 400, "")


  defp create_repo("maven", %{"group_id" => group_id, "artifact_id" => artifact_id}) do
    repo = %Enseada.Repository.Maven{
      group_id: group_id,
      artifact_id: artifact_id,
    }
    KV.put(:repos, "maven-#{group_id}-#{artifact_id}", repo)
  end

  defp create_repo(_, _), do: {:error, "unknown repo type"}

  defp scope(%{"group_id" => group_id, "artifact_id" => artifact_id}) do
    prefix = String.replace(group_id, ".", "/")
    "#{prefix}/#{artifact_id}"
  end
end
