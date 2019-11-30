import Config

# We don't run a server during test. If one is required,
# you can enable the server option below.
config :enseada,
       EnseadaWeb.Endpoint,
       http: [
         port: 4002
       ],
       server: false

# Print only warnings and errors during test
config :logger, level: :warn

config :enseada,
       :database,
       url: System.get_env("COUCHDB_URL") || "http://localhost:5984",
       username: System.get_env("COUCHDB_USER") || "enseada",
       password: System.get_env("COUCHDB_PASSWORD") || "enseada"
