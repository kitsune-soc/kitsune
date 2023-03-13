let types = ./kitsune/config/types.dhall

let makeSearchConfig =
      λ(url : Text) →
        { index_server = url, search_servers = [ url ] } : types.Search

in    { cache = types.Cache.Redis { redis_url = "redis://localhost:6379" }
      , database_url = "postgres://localhost/kitsune"
      , messaging = types.Messaging.InProcess
      , server =
            { frontend_dir = "./kitsune-fe/dist"
            , job_workers = 3
            , max_upload_size = 5 * 1024 * 1024
            , media_proxy_enabled = False
            , port = 5000
            , prometheus_port = 9000
            }
          : types.Server
      , search = makeSearchConfig "https://localhost:8080"
      , storage = types.Storage.Fs { upload_dir = "./uploads" }
      , url = { schema = "http", domain = "localhost:5000" } : types.Url
      }
    : types.Config
