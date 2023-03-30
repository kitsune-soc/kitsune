-- Example configuration for Kitsune
let types = ./kitsune/config/types.dhall

let makeSearchConfig =
      λ(url : Text) →
          types.Search.Kitsune { index_server = url, search_servers = [ url ] }
        : types.Search

in    { cache =
            types.Cache.Redis { redis_url = "redis://localhost:6379" }
          : types.Cache
      , database =
            { url = "postgres://localhost/kitsune", max_connections = 20 }
          : types.Database
      , instance =
            { name = "Kitsune"
            , description = "https://www.youtube.com/watch?v=6lnnPnr_0SU"
            , character_limit = 5000
            , registrations_open = True
            }
          : types.Instance
      , messaging = types.Messaging.InProcess
      , server =
            { frontend_dir = "./kitsune-fe/dist"
            , job_workers = 20
            , max_upload_size = 5 * 1024 * 1024
            , media_proxy_enabled = False
            , oidc = None types.Oidc
            , port = 5000
            , prometheus_port = 9000
            }
          : types.Server
      , search = types.Search.Meilisearch { instance_url = "http://localhost:7700", api_key = "xTJC-083-CMzIA0ga6gEXTpJKKafPe55JuxpMYzBkjc" }
      , storage = types.Storage.Fs { upload_dir = "./uploads" } : types.Storage
      , url = { scheme = "http", domain = "localhost:5000" } : types.Url
      }
    : types.Config
