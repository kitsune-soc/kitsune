let Cache = ./types/cache.dhall

let FsStorage = ./types/storage/fs.dhall

let Messaging = ./types/messaging.dhall

let RedisCache = ./types/cache/redis.dhall

let RedisMessaging = ./types/messaging/redis.dhall

let S3Storage = ./types/storage/s3.dhall

let Search = ./types/search.dhall

let Server = ./types/server.dhall

let Storage = ./types/storage.dhall

let Url = ./types/url.dhall

let Config =
      { cache : Cache
      , database_url : Text
      , messaging : Messaging
      , server : Server
      , search : Search
      , storage : Storage
      , url : Url
      }

in  { Cache
    , Config
    , FsStorage
    , Messaging
    , RedisCache
    , RedisMessaging
    , S3Storage
    , Search
    , Server
    , Storage
    , Url
    }
