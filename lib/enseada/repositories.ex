defmodule Enseada.Repositories do
  alias Enseada.Database
  alias Enseada.Mapper
  # TODO : implement ACL

  def list(opts \\ []) do
    {:ok, repos} = Database.list("repositories", include_docs: true)
    json = Mapper.db_list_to_json(repos)
    {:ok, json}
  end

  def create(repo, opts \\ []) do
    {:ok, repo} = Database.save("repositories", repo)
    json = Mapper.db_to_json(repo)
    {:ok, json}
  end
end
