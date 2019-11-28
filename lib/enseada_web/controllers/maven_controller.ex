defmodule EnseadaWeb.MavenController do
  use EnseadaWeb, :controller

  alias Enseada.Maven

  require Logger

  def resolve(conn, %{"glob" => glob}) do
    {filename, rest} = List.pop_at(glob, -1)
    scope = Enum.join(rest, "/")
    url = Maven.Storage.url({filename, scope})
    Logger.info("Sending file #{url}")
    redirect(conn, to: url)
  end

  def store(conn, %{"glob" => glob}) do
    {:ok, body, conn} = Plug.Conn.read_body(conn)
    {filename, rest} = List.pop_at(glob, -1)
    scope = Enum.join(rest, "/")

    Maven.Storage.store({%{filename: filename, binary: body}, scope})
    send_resp(conn, 200, "")
  end
end
