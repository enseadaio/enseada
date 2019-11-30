defmodule Enseada.Database do
  @moduledoc """
  Utility wrapper around CouchDB client
  """

  use GenServer
  require Logger

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  def create_database(name) do
    GenServer.call(__MODULE__, {:create_database, name})
  end

  def database_exists?(name) do
    GenServer.call(__MODULE__, {:database_exists, name})
  end

  def get_database(name) do
    GenServer.call(__MODULE__, {:get_database, name})
  end

  def get(db_name, id) do
    GenServer.call(__MODULE__, {:get, db_name, id})
  end

  def list(db_name, opts \\ []) do
    GenServer.call(__MODULE__, {:list, db_name, opts})
  end

  def save(db_name, document) do
    GenServer.call(__MODULE__, {:save, db_name, document})
  end

  def exists?(db_name, id) do
    GenServer.call(__MODULE__, {:exists, db_name, id})
  end

  def create_index(db_name, index, document, name, type, partial_filter_selector \\ %{}) do
    req = %{
      "index" => index,
      "ddoc" => document,
      "name" => name,
      "type" => type,
      "partial_filter_selector" => partial_filter_selector
    }

    GenServer.call(__MODULE__, {:create_index, db_name, req})
  end

  def find(db_name, query) do
    GenServer.call(__MODULE__, {:find, db_name, query})
  end

  def init(opts) do
    server_url =
      Keyword.get_lazy(
        opts,
        :url,
        fn ->
          config = Application.get_env(:enseada, :database, [])
          Keyword.get(config, :url, "http://localhost:5984")
        end
      )

    username =
      Keyword.get_lazy(
        opts,
        :username,
        fn ->
          config = Application.get_env(:enseada, :database, [])
          Keyword.get(config, :username)
        end
      )

    password =
      Keyword.get_lazy(
        opts,
        :password,
        fn ->
          config = Application.get_env(:enseada, :database, [])
          Keyword.get(config, :password)
        end
      )

    :couchdb.server_record(server_url, [{:basic_auth, {username, password}}])
  end

  def handle_call({:create_database, name}, _from, server) do
    reply =
      case :couchdb.database_record(server, name) do
        {:ok, db} -> :couchdb_databases.create(db)
        {:error, error} -> {:error, error}
      end

    {:reply, reply, server}
  end

  def handle_call({:database_exists, name}, _from, server) do
    {:reply, :couchdb_databases.exists(server, name), server}
  end

  def handle_call({:get_database, name}, _from, server) do
    {:reply, :couchdb.database_record(server, name), server}
  end

  def handle_call({:get, db_name, id}, _from, server) do
    reply =
      case :couchdb.database_record(server, db_name) do
        {:ok, db} -> :couchdb_documents.get(db, id)
        {:error, error} -> {:error, error}
      end

    {:reply, reply, server}
  end

  def handle_call({:list, db_name, opts}, _from, server) do
    include_docs = Keyword.get(opts, :include_docs, false)

    reply =
      case :couchdb.database_record(server, db_name) do
        {:ok, db} -> :couchdb_databases.all_docs(db, %{"include_docs" => include_docs})
        {:error, error} -> {:error, error}
      end

    {:reply, reply, server}
  end

  def handle_call({:save, db_name, document}, _from, server) do
    reply =
      case :couchdb.database_record(server, db_name) do
        {:ok, db} -> :couchdb_documents.save(db, document)
        {:error, error} -> {:error, error}
      end

    {:reply, reply, server}
  end

  def handle_call({:exists, db_name, id}, _from, server) do
    reply =
      case :couchdb.database_record(server, db_name) do
        {:ok, db} -> :couchdb_databases.exists(db, id)
        {:error, error} -> {:error, error}
      end

    {:reply, reply, server}
  end

  def handle_call({:create_index, db_name, index}, _from, server) do
    reply =
      case :couchdb.database_record(server, db_name) do
        {:ok, db} -> :couchdb_mango.index(db, index)
        {:error, error} -> {:error, error}
      end

    {:reply, reply, server}
  end

  def handle_call({:find, db_name, query}, _from, server) do
    reply =
      case :couchdb.database_record(server, db_name) do
        {:ok, db} -> :couchdb_mango.find(db, query)
        {:error, error} -> {:error, error}
      end

    {:reply, reply, server}
  end
end
