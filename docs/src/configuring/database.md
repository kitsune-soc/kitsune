# Database

Kitsune requires a PostgreSQL installation that it can connect to since we make usage of Postgres-specific features, such as their full-text search.

You can find instructions on creating a database (along with password-protected user) [here](https://medium.com/coding-blocks/creating-user-database-and-adding-access-on-postgresql-8bfcd2f4a91e).

> We supported SQLite in the past (before v0.0.1-pre.1), but the support has been dropped due to a high maintenance burden and rather little expected usage.

## Database URL structure

```
postgres://[Username]:[Password]@[DBMS host]:[Port]/[Database name]
```

### Example URL

```
postgres://database-user:password-here@localhost:5432/db-name-here
```

## Maximum connections

The `max-connections` setting defines how many connections the globally shared connection pool will open to the database server _at maximum_.  
What you should set this value to depends on many factors.

> We currently do not report any pool metrics via the Prometheus endpoint. This might be added in the future.

## TLS support

If you want to connect to a database using TLS, set the parameter `use-tls` to `true`.  
This setting is equivalent to `ssl_mode=full-verify` if you are looking for a PostgreSQL equivalent.

### Example

```toml
[database]
url = "postgres://kitsune:verysecure@localhost/kitsune_prod"
max-connections = 25
use-tls = true
```
