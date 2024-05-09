use async_trait::async_trait;
use eyre::WrapErr;
use http_body_util::BodyExt;
use http_compat::Compat;
use kitsune_config::{open_telemetry::Transport, Configuration};
use opentelemetry::trace::{noop::NoopTracer, Tracer};
use opentelemetry_http::{Bytes, HttpClient, HttpError, Request, Response};
use opentelemetry_otlp::{SpanExporterBuilder, WithExportConfig};
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

        let response = self.inner.execute(request.compat()).await?.into_inner();

        let (parts, body) = response.into_parts();
        let body = body.collect().await?.to_bytes();

        Ok(hyper::http::Response::from_parts(parts, body).compat())
    }
}

macro_rules! build_exporter {
    ($exporter_type:ty : $transport:expr, $http_client:expr, $endpoint:expr $(,)?) => {{
        let exporter: $exporter_type = match $transport {
            Transport::Grpc => opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint($endpoint)
                .into(),
            Transport::Http => opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint($endpoint)
                .with_http_client($http_client.clone())
                .into(),
        };

        exporter
    }};
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

        let trace_exporter = build_exporter!(
            SpanExporterBuilder:
            opentelemetry_config.tracing_transport,
            &http_client,
            opentelemetry_config.tracing_endpoint.as_str(),
        );

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(trace_exporter)
            .install_batch(Tokio)?;

        initialise_logging(tracer)?;
    } else {
        initialise_logging(NoopTracer::new())?;
    }

    Ok(())
}
