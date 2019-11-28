defmodule Enseada.Maven.Storage do
  use Arc.Definition
  @storage_dir Application.get_env(:arc, :storage_dir)

  def storage_dir(_, {_, scope}), do: "#{@storage_dir}/maven2/#{scope}"
end
