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
      , email = None types.Email
      , embed = None types.Embed
      , instance =
            { name = "Kitsune"
            , description = "https://www.youtube.com/watch?v=6lnnPnr_0SU"
            , character_limit = 5000
            , federation_filter =
                  types.FederationFilter.Deny { domains = [] : List Text }
                : types.FederationFilter
            , registrations_open = True
            }
          : types.Instance
      , job_queue = { redis_url = "redis://localhost" } : types.JobQueue
      , messaging = types.Messaging.InProcess
      , server =
            { frontend_dir = "./kitsune-fe/dist"
            , job_workers = 20
            , max_upload_size = 5 * 1024 * 1024
            , media_proxy_enabled = False
            , oidc = None types.Oidc
            , port = 5000
            , prometheus_port = 9000
            , request_timeout_sec = 60
            }
          : types.Server
      , search = makeSearchConfig "https://localhost:8081"
      , storage = types.Storage.Fs { upload_dir = "./uploads" } : types.Storage
      , url = { scheme = "http", domain = "localhost:5000" } : types.Url
      }
    : types.Config
