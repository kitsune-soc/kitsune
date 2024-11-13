use async_trait::async_trait;
use eyre::WrapErr;
use http_body_util::BodyExt;
use kitsune_config::{open_telemetry::Transport, Configuration};
use kitsune_core::consts::PROJECT_IDENTIFIER;
use opentelemetry::trace::{noop::NoopTracer, Tracer, TracerProvider};
use opentelemetry_http::{Bytes, HttpClient, HttpError, Request, Response};
use opentelemetry_otlp::{WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::runtime::Tokio;
use std::{env, fmt};
use tracing_error::ErrorLayer;
use tracing_opentelemetry::{OpenTelemetryLayer, PreSampledTracer};
use tracing_subscriber::{
    filter::{LevelFilter, Targets},
    layer::SubscriberExt,
    Layer as _, Registry,
};

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
    async fn send(&self, request: Request<Vec<u8>>) -> Result<Response<Bytes>, HttpError> {
        let (parts, body) = request.into_parts();
        let request = Request::from_parts(parts, body.into());

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
    let env_filter = env::var("RUST_LOG")
        .map_err(eyre::Report::from)
        .and_then(|targets| targets.parse().wrap_err("Failed to parse RUST_LOG value"))
        .unwrap_or_else(|_| Targets::default().with_default(LevelFilter::INFO));

    let subscriber = Registry::default()
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter))
        .with(ErrorLayer::default())
        .with(OpenTelemetryLayer::new(tracer));

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

        let tracer = opentelemetry_sdk::trace::TracerProvider::builder()
            .with_batch_exporter(trace_exporter, Tokio)
            .build()
            .tracer(PROJECT_IDENTIFIER);

        initialise_logging(tracer)?;
    } else {
        initialise_logging(NoopTracer::new())?;
    }

    Ok(())
}
