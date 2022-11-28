use self::handler::{oauth, posts, users, well_known};
use crate::state::State;
use axum::{http::StatusCode, routing::get_service, Router};
use std::io;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

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
    let frontend_dir = &state.config.frontend_dir;
    let frontend_index_path = {
        let mut tmp = frontend_dir.clone();
        tmp.push("index.html");
        tmp
    };

    let mut router = Router::new()
        .nest("/oauth", oauth::routes())
        .nest("/posts", posts::routes())
        .nest("/users", users::routes())
        .nest("/.well-known", well_known::routes())
        .nest_service(
            "/public",
            get_service(ServeDir::new("public")).handle_error(handle_error),
        )
        .nest_service(
            "/media",
            get_service(ServeDir::new(&state.config.upload_dir)).handle_error(handle_error),
        );

    #[cfg(feature = "mastodon-api")]
    {
        router = router.merge(handler::mastodon::routes());
    }

    let router = router
        .merge(graphql::routes(state.clone()))
        .fallback_service(
            get_service(ServeDir::new(frontend_dir).fallback(ServeFile::new(frontend_index_path)))
                .handle_error(handle_error),
        )
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    axum::Server::bind(&([0, 0, 0, 0], port).into())
        .serve(router.into_make_service())
        .await
        .unwrap();
}
