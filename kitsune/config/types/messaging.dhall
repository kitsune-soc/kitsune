let RedisMessaging = ./messaging/redis.dhall

in  < Redis : RedisMessaging | InProcess >
