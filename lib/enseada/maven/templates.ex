defmodule Enseada.Maven.Templates do
  require EEx

  @folder Application.app_dir(:enseada, "priv/templates")
  EEx.function_from_file(:defp, :render_metadata, "#{@folder}/maven-metadata.xml.eex", [
    :group_id,
    :artifact_id,
    :last_updated_timestamp
  ])

  def metadata(group_id, artifact_id) do
    render_metadata(group_id, artifact_id, last_updated_timestamp())
  end

  defp last_updated_timestamp() do
    DateTime.utc_now()
    |> format_date()
  end

  defp format_date(%DateTime{
         year: year,
         month: month,
         day: day,
         hour: hour,
         minute: minute,
         second: second
       }),
       do: "#{year}#{month}#{day}#{hour}#{minute}#{second}"
end
