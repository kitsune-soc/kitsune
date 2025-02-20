use crate::state::Zustand;
use axum::{
    body::HttpBody,
    response::{Html, IntoResponse},
    serve::ListenerExt,
};
use bytes::Bytes;
use color_eyre::eyre;
use http::{HeaderName, StatusCode};
use http_body_util::Either;
use kitsune_config::server;
use std::convert::Infallible;
use tokio::net::TcpListener;
use tower::{BoxError, Service, ServiceExt};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::{HttpMakeClassifier, MakeSpan, TraceLayer},
};

const FALLBACK_FALLBACK_INDEX: &str = include_str!("../../templates/fallback-fallback.html");

static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

#[cfg(feature = "graphql-api")]
mod graphql;
mod handler;
mod middleware;
#[cfg(feature = "mastodon-api")]
mod pagination;
mod responder;
mod util;

pub mod extractor;
pub mod router;

#[inline]
fn serve_frontend<B>(
    server_config: &server::Configuration,
) -> impl Service<
    http::Request<B>,
    Response = http::Response<impl HttpBody<Data = Bytes, Error = BoxError> + use<B>>,
    Error = Infallible,
    Future = impl Send + use<B>,
> + Clone
+ use<B>
where
    B: Send + 'static,
{
    let frontend_dir = &server_config.frontend_dir;
    let frontend_index_path = {
        let mut tmp = frontend_dir.to_string();
        tmp.push_str("/index.html");
        tmp
    };

    let handle_response = |response: http::Response<_>| {
        if response.status() == StatusCode::NOT_FOUND {
            (StatusCode::NOT_FOUND, Html(FALLBACK_FALLBACK_INDEX))
                .into_response()
                .map(Either::Left)
        } else {
            response.map(Either::Right)
        }
    };

    ServeDir::new(frontend_dir.as_str())
        .fallback(ServeFile::new(frontend_index_path))
        .map_future(move |result_fut| async move {
            let result = result_fut.await;
            result.map(handle_response)
        })
}

#[inline]
fn trace_layer<B>() -> TraceLayer<HttpMakeClassifier, impl MakeSpan<B> + Clone> {
    TraceLayer::new_for_http().make_span_with(|request: &http::Request<B>| {
        info_span!(
            "request",
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            request_id = ?request.headers().get(&X_REQUEST_ID).unwrap(),
        )
    })
}

#[instrument(skip_all, fields(port = %server_config.port))]
pub async fn run(
    state: Zustand,
    server_config: server::Configuration,
    shutdown_signal: crate::signal::Receiver,
) -> eyre::Result<()> {
    let router = router::create(state, &server_config)?;
    let listener = TcpListener::bind(("0.0.0.0", server_config.port))
        .await?
        .tap_io(|stream| {
            if let Err(error) = stream.set_nodelay(true) {
                trace!(?error, "failed to set TCP_NODELAY for connection");
            }
        });

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal.wait())
        .await?;

    Ok(())
}
