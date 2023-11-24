# OpenTelemetry

Kitsune can export its traces and metrics via the OpenTelemetry Protocol (or OTLP, for short).  

To push the data to an endpoint, add the following to your configuration:

```toml
[opentelemetry]
http-endpoint = "[URL of your HTTP endpoint]"
```

This pushes both, metrics and traces, to the same endpoint

> Note: We might change this in the future and allow to push metrics and traces to separate endpoints
