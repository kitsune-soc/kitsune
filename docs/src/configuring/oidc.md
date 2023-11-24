# OIDC (OpenID Connect)

> This feature is gated behind the `oidc` compile-time feature

OpenID Connect (OIDC) is a technology to provide Single Sign-On (SSO) that it shared across multiple services. 
This is useful if you, for example, want to run Kitsune together with a bunch of other services and don't want to maintain multiple logins.

In order to enable OIDC for your Kitsune instance, find the `oidc` parameter inside the `server` configuration section. 
Set this parameter to the following value:

```toml
[server.oidc]
server-url = "[Issuer URL]"
client-id = "[Kitsune's Client ID]"
client-secret = "[Kitsune's Client Secret]"

[server.oidc.store]
type = "in-memory" # "in-memory" or "redis"
#url = "redis://localhost" # If the configured type is "redis"
```

## Server URL

This is the URL of the issuer; this setting differs between OIDC solutions. Don't worry, Kitsune won't start up if this value is invalid.

## Client ID and Secret

These values are created on the dashboard of the OIDC solution you are using. 
Kitsune needs these to obtain an access token from the OIDC server to do introspection and obtain some information about the user.

## Store

The store is where the login state is stored in temporarily until the sign-in has completed.

### Supported backends

- `in-memory`: The state is stored in an in-memory LRU structure
- `redis`: The state is stored in a Redis instance

If you run multiple nodes, you should use Redis. Otherwise sign-ins can randomly fail.

## Kitsune-specific OIDC requirements

The OIDC server **must** return the following values in the claim:

- `preferred_username`
- `email`

> Keep the preferred username field unique. The username is used to identify the user inside Kitsune's database that's getting registered.  
> This might change in the future.
