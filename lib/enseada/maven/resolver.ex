defmodule Enseada.Maven.Resolver do
  def resolve_artifact({group, artifact, version, "maven-metadata.xml"}) do
    IO.puts("""
      Group: #{group}
      Artifact: #{artifact}
      Version: #{version}
      File: maven-metadata.xml
    """)

    file = """
    <metadata>
      <groupId>#{group}</groupId>
      <artifactId>#{artifact}</artifactId>
      <versioning>
        <latest>#{version}</latest>
        <release></release>
        <versions>
          <version>#{version}</version>
        </versions>
        <lastUpdated>20120321194127</lastUpdated>
      </versioning>
    </metadata>
    """

    {:ok, file}
  end

  def resolve_artifact({group, artifact, version, file}) do
    what = """
      Group: #{group}
      Artifact: #{artifact}
      Version: #{version}
      File: #{file}
    """

    {:ok, what}
  end

  def resolve_artifact(_), do: {:error, "invalid"}
end
