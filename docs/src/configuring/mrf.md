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

### `module-dir`

This configuration option tells Kitsune where to scan for WASM modules to load and compile.

```toml
[mrf]
module-dir = "mrf-modules"
```
