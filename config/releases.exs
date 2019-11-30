import Config

# Phoenix and HTTP config
secret_key_base =
  System.get_env("SECRET_KEY_BASE") ||
    raise """
    environment variable SECRET_KEY_BASE is missing.
    You can generate one by calling: mix phx.gen.secret
    """

host =
  System.get_env("PUBLIC_HOST") ||
    raise """
    environment variable PUBLIC_HOST is missing.
    Set it to a public accessible URL
    """

port = String.to_integer(System.get_env("PORT") || "4000")

ssl = System.get_env("SSL")

if ssl do
  key_file =
    System.get_env("SSL_KEY_PATH") ||
      raise """
      environment variable SSL_KEY_PATH is missing but SSL was activated.
      Set it to a valid SSL key file
      """

  cert_file =
    System.get_env("SSL_CERT_PATH") ||
      raise """
      environment variable SSL_CERT_PATH is missing but SSL was activated.
      Set it to a valid SSL certificate file
      """

  config :enseada,
         EnseadaWeb.Endpoint,
         url: [
           host: host,
           port: port
         ],
         https: [
           :inet6,
           port: port,
           cipher_suite: :strong,
           keyfile: key_file,
           certfile: cert_file
         ],
         force_ssl: [
           hsts: true
         ],
         secret_key_base: secret_key_base,
         server: true
else
  config :enseada,
         EnseadaWeb.Endpoint,
         url: [
           host: host,
           port: port
         ],
         http: [
           :inet6,
           port: port
         ],
         secret_key_base: secret_key_base,
         server: true
end

# Logger config
log_level = String.to_atom(System.get_env("LOG_LEVEL") || "info")

config :logger,
  level: log_level,
  backends: [:console]

# Storage config
config :waffle,
  asset_host: System.get_env("ASSET_HOST")

case System.get_env("STORAGE_PROVIDER") do
  "gcs" ->
    config :waffle,
      storage: Waffle.Storage.Google.CloudStorage,
      storage_dir: System.get_env("BUCKET_PREFIX") || "uploads",
      bucket: {:system, "GCS_BUCKET"}

    if System.get_env("GCS_JSON_CREDENTIALS") do
      config :goth, json: {:system, "GCS_JSON_CREDENTIALS"}
    end

  "s3" ->
    config :waffle,
      storage: Waffle.Storage.S3,
      storage_dir: System.get_env("BUCKET_PREFIX") || "uploads",
      bucket: {:system, "AWS_S3_BUCKET"}

    config :ex_aws,
      access_key_id: [{:system, "AWS_ACCESS_KEY_ID"}, :instance_role],
      secret_access_key: [{:system, "AWS_SECRET_ACCESS_KEY"}, :instance_role],
      region: System.get_env("AWS_REGION"),
      json_codec: Jason

    s3_url = System.get_env("AWS_S3_ENDPOINT")

    if s3_url do
      %{scheme: scheme, host: host, port: port} = URI.parse(s3_url)

      config :ex_aws,
             :s3,
             scheme: "#{scheme}://",
             host: host,
             port: port
    end

  p when p in ["local", nil] ->
    proto = if ssl, do: "https", else: "http"
    port = if port not in [80, 443], do: ":#{port}", else: ""
    asset_host = "#{proto}://#{host}#{port}"

    config :waffle,
      storage: Waffle.Storage.Local,
      storage_dir: "uploads",
      storage_dir_prefix: System.get_env("STORAGE_DIR") || "uploads",
      asset_host: asset_host
end

# Database config
database_url =
  System.get_env("COUCHDB_URL") ||
    raise """
    environment variable COUCHDB_URL is missing
    """

config :enseada,
       :database,
       url: database_url,
       username: System.get_env("COUCHDB_USER"),
       password: System.get_env("COUCHDB_PASSWORD")
