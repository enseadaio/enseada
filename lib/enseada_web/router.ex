defmodule EnseadaWeb.Router do
  use Plug.Router

  if Mix.env() == :dev do
    use Plug.Debugger
  end

  plug(Plug.RequestId)

  plug :redirect_ui
  plug(EnseadaWeb.ServeIndex, at: "/ui")

  plug(
    Plug.Static,
    at: "/ui",
    from: :enseada
  )

  plug(Plug.Logger)

  plug(
    Plug.Parsers,
    parsers: [:json],
    pass: ["*/*"],
    json_decoder: Jason
  )

  plug(Corsica, Application.get_env(:enseada, :cors))
  plug(:match)
  plug(:dispatch)

  get "/health" do
    send_resp(conn, 200, "ok")
  end

  match _ do
    send_resp(conn, 404, "Not Found")
  end

  defp redirect_ui(%{request_path: "/ui"} = conn, _) do
    conn
    |> put_resp_header("location", "/ui/")
    |> send_resp(301, "")
  end
  defp redirect_ui(conn, _), do: conn
end
