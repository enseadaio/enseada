defmodule Enseada.Maven.Storage do
  use Arc.Definition

  def storage_dir(_, {_, scope}), do: "uploads/maven2/#{scope}"
end
