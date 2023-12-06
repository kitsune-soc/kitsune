use self::{
    handler::{
        confirm_account, custom_emojis, media, nodeinfo, oauth, posts, public, users, well_known,
    },
    openapi::api_docs,
};
use crate::state::Zustand;
use axum::{extract::DefaultBodyLimit, Router};
use kitsune_config::server;
use miette::{Context, IntoDiagnostic};
use std::time::Duration;
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tower_x_clacks_overhead::XClacksOverheadLayer;
use utoipa_swagger_ui::SwaggerUi;

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

pub fn create_router(
    state: Zustand,
    server_config: &server::Configuration,
) -> miette::Result<Router> {
    let frontend_dir = &server_config.frontend_dir;
    let frontend_index_path = {
        let mut tmp = frontend_dir.to_string();
        tmp.push_str("/index.html");
        tmp
    };

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
        .fallback_service(
            ServeDir::new(frontend_dir.as_str()).fallback(ServeFile::new(frontend_index_path)),
        );

    if !server_config.clacks_overhead.is_empty() {
        let clacks_overhead_layer =
            XClacksOverheadLayer::new(server_config.clacks_overhead.iter().map(AsRef::as_ref))
                .into_diagnostic()
                .wrap_err("Invalid clacks overhead values")?;

        router = router.layer(clacks_overhead_layer);
    }

    Ok(router
        .layer(CatchPanicLayer::new())
        .layer(CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(server_config.max_upload_size))
        .layer(TimeoutLayer::new(Duration::from_secs(
            server_config.request_timeout_secs,
        )))
        .layer(TraceLayer::new_for_http())
        .with_state(state))
}

#[instrument(skip_all, fields(port = %server_config.port))]
pub async fn run(state: Zustand, server_config: server::Configuration) -> miette::Result<()> {
    let router = create_router(state, &server_config)?;

    axum::Server::bind(&([0, 0, 0, 0], server_config.port).into())
        .serve(router.into_make_service())
        .await
        .into_diagnostic()?;

    Ok(())
}
