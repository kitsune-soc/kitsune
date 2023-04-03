let Cache = ./types/cache.dhall

let Database = ./types/database.dhall

let FsStorage = ./types/storage/fs.dhall

let Instance = ./types/instance.dhall

let Kitsune = ./types/search/kitsune.dhall

let Meilisearch = ./types/search/meilisearch.dhall

let Messaging = ./types/messaging.dhall

let Oidc = ./types/oidc.dhall

let RedisCache = ./types/cache/redis.dhall

let RedisMessaging = ./types/messaging/redis.dhall

let S3Storage = ./types/storage/s3.dhall

let Search = ./types/search.dhall

let Server = ./types/server.dhall

let Storage = ./types/storage.dhall

let Url = ./types/url.dhall

let Config =
      { cache : Cache
      , database : Database
      , instance : Instance
      , messaging : Messaging
      , server : Server
      , search : Search
      , storage : Storage
      , url : Url
      }

in  { Cache
    , Config
    , Database
    , FsStorage
    , Instance
    , Kitsune
    , Meilisearch
    , Messaging
    , Oidc
    , RedisCache
    , RedisMessaging
    , S3Storage
    , Search
    , Server
    , Storage
    , Url
    }
