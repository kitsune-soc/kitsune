# MRF (Message Rewrite Facility)

The idea of a message rewrite facility was originally popularized by Pleroma/Akkoma.  
Essentially it enables you to add small filters/transformers into your ActivityPub federation.

The MRF module sits at the very beginning of processing an incoming activity.  
In this position, the MRF can transform and reject activities based on criteria defined by the developers of the module.

For example, you can use it to:

- detect spam
- mark media attachments as sensitive
- nya-ify every incoming post

## Configuration

### `allocation-strategy`

⚠ You usually don't need to touch this setting!

It has two possible values:

- `on-demand`
- `pooling`

By default it is set to `pooling` which will offer the best throughput since Kitsune instantiates short-lived WASM modules, where pooled allocation can improve performance.

If Kitsune crashes on startup complaining about `failed to create stack pool mapping`, you might want to try changing this setting to `on-demand`.

### `module-dir`

This configuration option tells Kitsune where to scan for WASM modules to load and compile.

```toml
[mrf]
module-dir = "mrf-modules"
```

### `artifact-cache`

This configuration option tells Kitsune to cache compiled versions of the WASM components on-disk to improve startup times
by not having to recompile the components every time Kitsune starts up.

Simply omitting this configuration disables the caching.

```toml
[mrf.artifact-cache]
path = "./artifact-cache"
```

> Note: DO NOT modify the files inside the artifact cache. You can delete the cache to reclaim space, but DO NOT modify the files.
> The files inside the cache are treated by Kitsune as trusted executables. Modifying them may lead to unexpected behaviour and crashes.

### `storage`

Kitsune provides MRF modules with scoped key-value storages to allow them to persist data across runs for things like counters or spam lists.

We provide multiple backends here:

#### Redis

```toml
[mrf.storage]
type = "redis"
url = "redis://localhost"
pool-size = 5
```

#### Filesystem

```toml
[mrf.storage]
type = "fs"
path = "./mrf-storage"
```

### `module-config`

WASM MRFs can have configuration passed to them upon invocation. In Kitsune you configure them via key-value pairs inside your configuration as follows:

```toml
[mrf.module-config]
module-name = "my configuration"
```

The names that are used to select the configuration are sourced from the module manifest.
