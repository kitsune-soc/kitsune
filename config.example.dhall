-- Example configuration for Kitsune
let types = ./kitsune/config/types.dhall

let makeSearchConfig =
      λ(url : Text) →
        { index_server = url, search_servers = [ url ] } : types.Search

in    { cache =
            types.Cache.Redis { redis_url = "redis://localhost:6379" }
          : types.Cache
      , database_url = "postgres://localhost/kitsune"
      , instance =
            { name = "Kitsune ActivityPub server lmao"
            , description = "https://www.youtube.com/watch?v=6lnnPnr_0SU"
            , character_limit = 5000
            }
          : types.Instance
      , messaging = types.Messaging.InProcess
      , server =
            { frontend_dir = "./kitsune-fe/dist"
            , job_workers = 20
            , max_upload_size = 5 * 1024 * 1024 {- Maximum upload size in bytes -}
            , media_proxy_enabled = False {- This will proxy all remote attachments through Kitsune, enabling caching and better privacy for the users -}
            , port = 5000
            , prometheus_port = 9000
            }
          : types.Server
      , search = makeSearchConfig "https://localhost:8080"
      , storage = types.Storage.Fs { upload_dir = "./uploads" } : types.Storage
      , url = { scheme = "http", domain = "localhost:5000" } : types.Url
      }
    : types.Config
