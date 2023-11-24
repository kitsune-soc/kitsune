# Search

Kitsune has a number of search backends, each different from the other, to best fit your specific needs. 
We want to give you a brief overview over the available backends.

## No Search

```toml
[search]
type = "none"
```

This completely disables the search on your instance. Finding posts and accounts is now only possible via direct links and handles.

## SQL-based Search

```toml
[search]
type = "sql"
```

This runs searches on your database directly. The quality is actually not too bad and uses automatic language detection to make full-text searches relevant.

## Meilisearch

> You need to compile Kitsune with the `meilisearch` feature flag to enable this feature

```toml
[search]
type = "meilisearch"
instance-url = "[URL of your Meilisearch instance]"
api-key = "[API key to access your Meilisearch instance]"
```

This instructs Kitsune to use [Meilisearch](https://www.meilisearch.com/) as the search engine. Meilisearch provides incredibly fast, high-quality full-text search.  
Meilisearch also has a cloud offering, making this the easiest high-quality search to use with Kitsune.
