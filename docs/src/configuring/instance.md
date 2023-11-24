# Instance

Kitsune has a number of configurations that change how your instance works.

```toml
[instance]
name = "Kitsune"
description = "https://www.youtube.com/watch?v=6lnnPnr_0SU"
character-limit = 5000
registrations-open = true
```

## `name`

This changes the name of the instance displayed on the landing page and returned via instance metadata endpoints (such as Mastodon's `/api/v1/instance`).

## `description`

Similar to `name`, this setting adjusts the description on the landing page and the one returned via the metadata endpoints.

> **Note**: This field is interpreted as raw HTML

## `character-limit`

This setting sets the character limit specific to your instance.

## `registrations-open`

Determines whether your instance accepts new users or not. When set to `false`, the registration APIs will return a failure code.

## `webfinger-domain`

This enables you to host your `.well-known/webfinger` resource on your main domain (i.e. `example.com`) and the web UI and inboxes on a subdomain (i.e. `kitsune.example.com`).  
The advantage of this configuration is that your handle can be `@me@example.com`, while the account is hosted on `fedi.example.com`.

### Example value

```toml
webfinger-domain = "example.com"
```
