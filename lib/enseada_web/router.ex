defmodule EnseadaWeb.Router do
  use EnseadaWeb, :router

  pipeline :browser do
    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_flash
    plug :protect_from_forgery
    plug :put_secure_browser_headers
  end

  pipeline :api do
    plug :accepts, ["json"]
  end

  scope "/ui", EnseadaWeb do
    pipe_through :browser

    get "/*_rest", UIController, :index
  end

  scope "/maven2", EnseadaWeb do
    get "/*glob", MavenController, :resolve
    put "/*glob", MavenController, :store
  end

  scope "/api", EnseadaWeb.Api do
    pipe_through :api

    get "/test", RepoController, :test
    get "/repositories", RepoController, :index
    post "/repositories", RepoController, :create
  end
end
