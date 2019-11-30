defmodule Enseada.Mapper do
  alias Enseada.Helpers

  def db_to_json(%{
        _id: id,
        group_id: group_id,
        artifact_id: artifact_id,
        name: name,
        type: type
      }),
      do: %{
        id: id,
        group_id: group_id,
        artifact_id: artifact_id,
        name: name,
        type: type
      }

  def db_to_json(repo) when is_map(repo) do
    repo
    |> Helpers.atomize_keys()
    |> db_to_json()
  end

  def db_list_to_json(%{
        offset: offset,
        rows: rows,
        total_rows: total_rows
      }) do
    items =
      rows
      |> Enum.map(fn %{doc: doc} -> doc end)
      |> Enum.map(&db_to_json/1)

    %{
      offset: offset,
      items: items,
      total_items: total_rows
    }
  end

  def db_list_to_json(repos) when is_map(repos) do
    repos
    |> Helpers.atomize_keys()
    |> db_list_to_json()
  end
end
