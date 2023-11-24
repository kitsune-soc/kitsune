# Federation filter

Kitsune has a basic federation filter. It has two main modes of operation:

- Allowlist-based
- Denylist-based

The domain list supports [globbing](https://en.wikipedia.org/wiki/Glob_(programming)), so can, among other things, define wildcard blocks.

## Allowlist

As the name might suggests, this mode allows instance administrators to define a set list of domains the server is allowed to interact with.  
Federation attempts from any other server are rejected.

### Configuration example

```toml
[instance.federation-filter]
type = "allow" 
domains = ["*.myfriends.com", "cool-instan.ce"]
```

## Denylist

This is the opposite of the allowlist-based federation. In this mode, Kitsune generally accepts federation attempts from any instance *except* the ones defined in the domain list.

### Configuration example

```toml
[instance.federation-filter]
type = "deny"
domains = ["*.badstuff.com", "mean-people.biz"]
```
