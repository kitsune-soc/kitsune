use async_trait::async_trait;
use eyre::WrapErr;
use http_body_util::BodyExt;
use kitsune_config::{Configuration, open_telemetry::Transport};
use kitsune_core::consts::PROJECT_IDENTIFIER;
use opentelemetry::trace::{Tracer, TracerProvider, noop::NoopTracer};
use opentelemetry_http::{Bytes, HttpClient, HttpError, Request, Response};
use opentelemetry_otlp::{WithExportConfig, WithHttpConfig};
use std::fmt;
use tracing_error::ErrorLayer;
use tracing_opentelemetry::{OpenTelemetryLayer, PreSampledTracer};
use tracing_subscriber::{Layer as _, Registry, filter::LevelFilter, layer::SubscriberExt};

#[derive(Clone)]
struct HttpClientAdapter {
    inner: kitsune_http_client::Client,
}

impl fmt::Debug for HttpClientAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl HttpClient for HttpClientAdapter {
    async fn send_bytes(&self, request: Request<Bytes>) -> Result<Response<Bytes>, HttpError> {
        let request = request.map(Into::into);
        let response = self.inner.execute(request).await?.into_inner();

        let (parts, body) = response.into_parts();
        let body = body.collect().await?.to_bytes();

        Ok(hyper::http::Response::from_parts(parts, body))
    }
}

fn initialise_logging<T>(tracer: T) -> eyre::Result<()>
where
    T: Tracer + PreSampledTracer + Send + Sync + 'static,
{
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    let subscriber = Registry::default()
        .with(console_subscriber::spawn())
        .with(ErrorLayer::default())
        .with(OpenTelemetryLayer::new(tracer))
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter));

    tracing::subscriber::set_global_default(subscriber)
        .wrap_err("Couldn't install the global tracing subscriber")?;

    Ok(())
}

pub fn initialise(config: &Configuration) -> eyre::Result<()> {
    if let Some(ref opentelemetry_config) = config.opentelemetry {
        let http_client = HttpClientAdapter {
            inner: kitsune_http_client::Client::default(),
        };

        let trace_exporter = match opentelemetry_config.tracing_transport {
            Transport::Grpc => opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(opentelemetry_config.tracing_endpoint.as_str())
                .build()?,
            Transport::Http => opentelemetry_otlp::SpanExporter::builder()
                .with_http()
                .with_endpoint(opentelemetry_config.tracing_endpoint.as_str())
                .with_http_client(http_client.clone())
                .build()?,
        };

        let tracer = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(trace_exporter)
            .build()
            .tracer(PROJECT_IDENTIFIER);

        initialise_logging(tracer)?;
    } else {
        initialise_logging(NoopTracer::new())?;
    }

    Ok(())
}
