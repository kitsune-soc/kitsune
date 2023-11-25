# Cache

Computing things from scratch can be pretty expensive, that's where caching comes in.  
To best fit for your specific setup, Kitsune has multiple caching backends:

## No Cache

```toml
[cache]
type = "none"
```

This is the simplest of all caching modes. It just doesn't cache anything at all and utilises Kitsune's no-op cache. 
Pretty much only desirable if you are debugging other caches for invalidation issues (or if you have *heavy* memory constraints and no way to get your hands on a Redis instance).

## In-Memory Cache

```toml
[cache]
type = "in-memory"
```

This tells Kitsune to cache data directly into its memory. The cache is bounded so it doesn't make you run out of memory.

## Redis Cache

```toml
[cache]
type = "redis"
redis-url = "redis://[Your Redis instance]"
```

This tells Kitsune to cache data via expiring keys into the configured Redis instance.  
This is the optimal configuration for setups where you have multiple Kitsune nodes running at the same time.
