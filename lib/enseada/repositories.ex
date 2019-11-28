defmodule Enseada.Repositories do
  require Logger

  def start() do
    {:ok, server} = :couchdb.server_record("http://localhost:5984", [{:basic_auth, {"enseada", "enseada"}}])
    {:ok, info} = :couchdb_server.info(server)
    {:ok, db} = db(server)
    {:ok, doc} = :couchdb_documents.save(db, %{hello: :world})
    Logger.info("Saved document: #{inspect(doc)}")
    {:ok, docs} = :couchdb_databases.all_docs(db)
    Logger.info("All Docs: #{inspect(docs)}")
  end

  defp db(server) do
    {:ok, db} = :couchdb.database_record(server, "repositories")
    if :couchdb_databases.exists(server, "repositories") do
      {:ok, db}
    else
      :couchdb_databases.create(db)
    end
  end
end