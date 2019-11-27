defmodule EnseadaWeb.ServeIndex do
  @behaviour Plug
  @moduledoc """
  Ported from https://github.com/mbuhot/plug_static_index_html
  Serves `index.html` pages for requests to paths without a filename in Plug applications.
  """

  @doc ~S"""
  Initialize plug options
   - at: The request path to reach for static assets, defaults to "/"
   - default_file: Filename to serve when request path is a directory, defaults to "index.html"
  ## Example
      iex> Plug.Static.IndexHtml.init(at: "/doc")
      [matcher: ~r|^/doc/(.*/)?$|, default_file: "index.html"]
  """
  def init([]), do: init(at: "/")
  def init(at: path), do: init(at: path, default_file: "index.html")

  def init(at: path, default_file: filename) do
    path_no_slash = String.trim_trailing(path, "/")
    [matcher: ~r|^#{path_no_slash}/(.*/)?$|, default_file: filename]
  end

  @doc """
  Invokes the plug, adding default_file to request_path and path_info for directory paths
  ## Example
      iex> opts = Plug.Static.IndexHtml.init(at: "/doc")
      iex> conn = %Plug.Conn{request_path: "/doc/a/", path_info: ["doc", "a"]}
      iex> Plug.Static.IndexHtml.call(conn, opts) |> Map.take([:request_path, :path_info])
      %{path_info: ["doc", "a", "index.html"], request_path: "/doc/a/index.html"}
  """
  def call(conn, matcher: pattern, default_file: filename) do
    if String.match?(conn.request_path, pattern) do
      %{
        conn
        | request_path: "#{conn.request_path}#{filename}",
          path_info: conn.path_info ++ [filename]
      }
    else
      conn
    end
  end
end
