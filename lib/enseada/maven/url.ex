defmodule Enseada.Maven.Url do
  require Logger

  def parse_glob([]), do: []

  def parse_glob(glob) do
    reversed = Enum.reverse(glob)
    detect_glob_structure(reversed)
  end

  defp detect_glob_structure(["maven-metadata." <> type, version, artifact | group_parts]) do
    group = compose_group(group_parts)

    Logger.info(
      "Detected versioned, metadata file: #{
        inspect({group, artifact, version, "maven-metadata.#{type}"})
      }"
    )

    {group, artifact, version, "maven-metadata.#{type}"}
  end

  defp detect_glob_structure(["maven-metadata." <> type, artifact | group_parts]) do
    group = compose_group(group_parts)
    Logger.info("Detected metadata file: #{inspect({group, artifact, "maven-metadata.#{type}"})}")
    {group, artifact, "maven-metadata.#{type}"}
  end

  defp detect_glob_structure([file, version, artifact | group_parts]) do
    group = compose_group(group_parts)
    Logger.info("Detected regular file: #{inspect({group, artifact, version, file})}")
    {group, artifact, version, file}
  end

  defp compose_group([]), do: ""

  defp compose_group(group_parts) do
    group_parts
    |> Enum.reverse()
    |> Enum.join(".")
  end
end
