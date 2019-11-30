defmodule Enseada.Maven.Storage do
  use Waffle.Definition

  def storage_dir(_, {_, scope}) do
    dir = Application.get_env(:waffle, :storage_dir)
    "#{dir}/maven2/#{scope}"
  end
end
