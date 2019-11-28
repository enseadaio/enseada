defmodule Enseada.Repository.Maven do
  @derive Jason.Encoder
  defstruct [:group_id, :artifact_id]
end