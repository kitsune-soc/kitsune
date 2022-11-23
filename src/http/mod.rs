use self::handler::{oauth, posts, users, well_known};
use crate::state::State;
use axum::{http::StatusCode, routing::get_service, Extension, Router};
use std::io;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

pub mod graphql;

mod extractor;
mod handler;

#[allow(clippy::unused_async)]
async fn handle_error(err: io::Error) -> StatusCode {
    error!(error = %err, "Static file handler failed");
    StatusCode::INTERNAL_SERVER_ERROR
}

#[instrument(skip(state))]
pub async fn run(state: State, port: u16) {
    let mut router = Router::new()
        .nest("/oauth", oauth::routes())
        .nest("/posts", posts::routes())
        .nest("/users", users::routes())
        .nest("/.well-known", well_known::routes())
        .merge(graphql::routes(state.clone()))
        .fallback(get_service(ServeDir::new("public")).handle_error(handle_error));

    #[cfg(feature = "mastodon-api")]
    {
        router = router.merge(handler::mastodon::routes());
    }

    router = router
        .layer(Extension(state))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&([0, 0, 0, 0], port).into())
        .serve(router.into_make_service())
        .await
        .unwrap();
}
