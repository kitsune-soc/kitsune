use self::{
    handler::{
        confirm_account, custom_emojis, media, nodeinfo, oauth, posts, public, users, well_known,
    },
    openapi::api_docs,
};
use crate::state::Zustand;
use axum::{
    extract::DefaultBodyLimit,
    response::{Html, IntoResponse, Response},
    Router,
};
use color_eyre::eyre::{self, Context};
use cursiv::CsrfLayer;
use http::{HeaderName, StatusCode};
use kitsune_config::server;
use std::{convert::Infallible, time::Duration};
use tokio::net::TcpListener;
use tower::{Service, ServiceExt};
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::{HttpMakeClassifier, MakeSpan, TraceLayer},
};
use tower_stop_using_brave::StopUsingBraveLayer;
use tower_x_clacks_overhead::XClacksOverheadLayer;
use utoipa_swagger_ui::SwaggerUi;

static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

const FALLBACK_FALLBACK_INDEX: &str = r##"
<!DOCTYPE html>
<html>
  <head>
    <title>Welcome to Kitsune!</title>
    <style>
      html {
        color-scheme: light dark;
      }
      body {
        width: 35em;
        margin: 0 auto;
        font-family: Tahoma, Verdana, Arial, sans-serif;
      }
    </style>
  </head>
  <body>
    <h1>Welcome to Kitsune!</h1>
    <p>
      If you see this page, the Kitsune fediverse server is successfully
      installed and working. Further configuration is required.
    </p>

    <p>
      For online documentation and support please refer to
      <a href="http://joinkitsune.org/">joinkitsune.org</a>.<br />
      Commercial support is available at
      <a href="#">fuckall nowhere</a>.
    </p>

    <p><em>Thank you for using Kitsune.</em></p>
  </body>
</html>
"##;

#[cfg(feature = "graphql-api")]
mod graphql;
mod handler;
mod middleware;
mod openapi;
#[cfg(feature = "mastodon-api")]
mod pagination;
mod responder;
mod util;

pub mod extractor;

#[inline]
fn serve_frontend<B>(
    server_config: &server::Configuration,
) -> impl Service<http::Request<B>, Response = Response, Error = Infallible, Future = impl Send> + Clone
where
    B: Send + 'static,
{
    let frontend_dir = &server_config.frontend_dir;
    let frontend_index_path = {
        let mut tmp = frontend_dir.to_string();
        tmp.push_str("/index.html");
        tmp
    };

    let serve_frontend =
        ServeDir::new(frontend_dir.as_str()).fallback(ServeFile::new(frontend_index_path));

    serve_frontend.map_future(|result_fut| async move {
        let result = result_fut.await;
        result.map(|response| {
            if response.status() == StatusCode::NOT_FOUND {
                (StatusCode::NOT_FOUND, Html(FALLBACK_FALLBACK_INDEX)).into_response()
            } else {
                response.map(axum::body::Body::new)
            }
        })
    })
}

#[inline]
fn trace_layer<B>() -> TraceLayer<HttpMakeClassifier, impl MakeSpan<B> + Clone> {
    TraceLayer::new_for_http().make_span_with(|request: &http::Request<B>| {
        debug_span!(
            "request",
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            request_id = ?request.headers().get(&X_REQUEST_ID).unwrap(),
        )
    })
}

pub fn create_router(
    state: Zustand,
    server_config: &server::Configuration,
) -> eyre::Result<Router> {
    // This warning will come up if the server is compiled without the Mastodon API compatibility
    #[allow(unused_mut)]
    let mut router = Router::new()
        .nest("/confirm-account", confirm_account::routes())
        .nest("/emojis", custom_emojis::routes())
        .nest("/media", media::routes())
        .nest("/nodeinfo", nodeinfo::routes())
        .nest(
            "/oauth",
            oauth::routes().layer(axum::middleware::from_fn(middleware::json_to_urlencoded)),
        )
        .nest("/posts", posts::routes())
        .nest("/users", users::routes())
        .nest("/.well-known", well_known::routes())
        .nest("/public", public::routes());

    #[cfg(feature = "oidc")]
    {
        router = router.nest("/oidc", handler::oidc::routes());
    }

    #[cfg(feature = "graphql-api")]
    {
        router = router.merge(graphql::routes(state.clone()));
    }

    #[cfg(feature = "mastodon-api")]
    {
        router = router.merge(handler::mastodon::routes());
    }

    router = router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", api_docs()))
        .fallback_service(serve_frontend(server_config));

    if !server_config.clacks_overhead.is_empty() {
        let clacks_overhead_layer =
            XClacksOverheadLayer::new(server_config.clacks_overhead.iter().map(AsRef::as_ref))
                .wrap_err("Invalid clacks overhead values")?;

        router = router.layer(clacks_overhead_layer);
    }

    if server_config.deny_brave_browsers {
        router = router.layer(StopUsingBraveLayer::default());
    }

    Ok(router
        .layer(CatchPanicLayer::new())
        .layer(CorsLayer::permissive())
        .layer(CsrfLayer::generate()) // TODO: Make this configurable instead of random
        .layer(DefaultBodyLimit::max(
            server_config.max_upload_size.to_bytes() as usize,
        ))
        .layer(TimeoutLayer::new(Duration::from_secs(
            server_config.request_timeout_secs,
        )))
        .layer(trace_layer())
        .layer(PropagateRequestIdLayer::new(X_REQUEST_ID.clone()))
        .layer(SetRequestIdLayer::new(
            X_REQUEST_ID.clone(),
            MakeRequestUuid,
        ))
        .with_state(state))
}

#[instrument(skip_all, fields(port = %server_config.port))]
pub async fn run(
    state: Zustand,
    server_config: server::Configuration,
    shutdown_signal: crate::signal::Receiver,
) -> eyre::Result<()> {
    let router = create_router(state, &server_config)?;
    let listener = TcpListener::bind(("0.0.0.0", server_config.port)).await?;

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal.wait())
        .await?;

    Ok(())
}
