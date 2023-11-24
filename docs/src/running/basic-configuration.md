# Basic configuration

Kitsune is using the [TOML configuration format](https://toml.io) to configure the main application and the job runner.  
The syntax itself is easy to grasp and is essentially and extended INI format.

> The auxiliary services/CLI tools are using environment variables at the moment. 
> Note that this might change in the future.

> Example configurations for external programs can be found in `kitsune/contrib`.

> The example config for Kitsune can be found in the root directory titled "config.example.toml", move it wherever you like, and feel free to rename it. 

The simplest Kitsune configuration looks like this:

```toml
[cache]
type = "in-memory"

[database]
url = "postgres://localhost/kitsune"
max-connections = 20

[instance]
name = "Kitsune"
description = "https://www.youtube.com/watch?v=6lnnPnr_0SU"
character-limit = 5000
registrations-open = true

[instance.federation-filter]
type = "deny"
domains = []

[job-queue]
redis-url = "redis://localhost"
num-workers = 20

[messaging]
type = "in-process"

[server]
frontend-dir = "./kitsune-fe/dist"
max-upload-size = 20242880          # 5MB
media-proxy-enabled = false
port = 5000
request-timeout-secs = 60

[search]
type = "sql"

[storage]
type = "fs"
upload-dir = "./uploads"

[url]
scheme = "https"
domain = "kitsune.example.com"
```

To successfully deploy the application, make sure you **at least** change the following sections to your own:

- `domain`

  - Domain of your instance. Used to build the URLs of your activities.
  - This is a *very important* setting and **cannot** be changed after the first setup.

- `database`

  - Specifically the `url` parameter. Refer to [database section](../configuring/database) page for the expected format.

- `job-queue`

  - Specifically the `redis-url` parameter. Refer to the [job scheduler](../configuring/job-scheduler) page for the expected format.

To run the application use the following command:

```bash
./kitsune -c [path-to-config-file]
```

In order to read up on all the possible configurations, check out the "Configuration" section.
