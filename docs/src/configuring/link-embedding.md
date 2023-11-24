# Link embedding

Kitsune has the ability to show link previews as so-called "embed cards".  
We use [Lantern Chat's `embed-service`](https://github.com/Lantern-chat/embed-service/) to provide this functionality.

To use it with Kitsune, run the `embed-service` and add the following configuration to Kitsune's configuration file:

```toml
[embed]
service-url = "[URL to the embed service]"
```
