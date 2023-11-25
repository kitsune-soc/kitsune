# Messaging

Kitsune uses messaging services to exchange events for cache invalidation, notification delivery, etc.  
To offer flexibility and to make self-hosting as easy as possible, we have multiple such backends.

## Redis backend

This is backend you should choose when running multiple nodes in parallel. This then uses Redis' PubSub feature to exchange messages.

### Configuration example

```toml
[messaging]
type = "redis"
redis-url = "redis://localhost:6379"
```

## In-process

This backend is optimal for small self-contained installations. It uses Tokio's broadcast channel internally.

### Configuration example

```toml
[messaging]
type = "in-process"
