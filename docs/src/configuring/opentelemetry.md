# OpenTelemetry

Kitsune can export its traces and metrics via the OpenTelemetry Protocol (or OTLP, for short).  

To push the data to an endpoint, add the following to your configuration:

```toml
[opentelemetry]
# Where Kitsune pushes metrics (eg. Prometheus)
metrics-transport = "http" # "http" or "grpc"
metrics-endpoint = "https://localhost:5050/metrics-endpoint"
# Where Kitsune pushes traces (eg. Jaeger)
tracing-transport = "http" # "http" or "grpc"
tracing-endpoint = "https://localhost:5050/tracing-endpoint"
```
