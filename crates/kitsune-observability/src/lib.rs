use eyre::Context;
use kitsune_config::Configuration;
use metrics_opentelemetry::OpenTelemetryRecorder;
use opentelemetry::{metrics::MeterProvider, trace::Tracer};
use std::env;
use tracing_error::ErrorLayer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{
    filter::{LevelFilter, Targets},
    layer::SubscriberExt,
    Layer, Registry,
};

fn initialise_metrics<S, M>(config: &Configuration, meter_provider: M) -> impl Layer<S>
where
    S: for<'a> tracing_subscriber::registry::LookupSpan<'a> + tracing::Subscriber,
    M: MeterProvider,
{
    use metrics_tracing_context::{MetricsLayer, TracingContextLayer};
    use metrics_util::layers::Layer as _;

    let recorder = TracingContextLayer::all()
        .layer(OpenTelemetryRecorder::new(meter_provider.meter("kitsune")));
    metrics::set_boxed_recorder(Box::new(recorder)).unwrap();

    MetricsLayer::new()
}

fn initialise_logging(config: &Configuration) -> eyre::Result<()> {
    let env_filter = env::var("RUST_LOG")
        .map_err(eyre::Report::from)
        .and_then(|targets| targets.parse().context("Failed to parse RUST_LOG value"))
        .unwrap_or_else(|_| Targets::default().with_default(LevelFilter::INFO));

    let subscriber = Registry::default()
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter))
        .with(ErrorLayer::default())
        .with(OpenTelemetryLayer::new());

    let subscriber = subscriber.with(initialise_metrics(config));

    tracing::subscriber::set_global_default(subscriber)
        .context("Couldn't install the global tracing subscriber")?;

    Ok(())
}
