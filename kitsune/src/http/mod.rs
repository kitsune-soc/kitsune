use self::handler::{media, nodeinfo, oauth, posts, users, well_known};
use crate::state::Zustand;
use axum::{routing::get_service, Router};
use axum_prometheus::PrometheusMetricLayer;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

mod cond;
mod extractor;
mod graphql;
mod handler;
mod page;

#[instrument(skip(state))]
pub async fn run(state: Zustand, port: u16) {
    let frontend_dir = &state.config.frontend_dir;
    let frontend_index_path = {
        let mut tmp = frontend_dir.clone();
        tmp.push("index.html");
        tmp
    };

    // This warning will come up if the server is compiled without the Mastodon API compatibility
    #[allow(unused_mut)]
    let mut router = Router::new()
        .nest("/media", media::routes())
        .nest("/nodeinfo", nodeinfo::routes())
        .nest("/oauth", oauth::routes())
        .nest("/posts", posts::routes())
        .nest("/users", users::routes())
        .nest("/.well-known", well_known::routes())
        .nest_service("/public", get_service(ServeDir::new("public")));

    #[cfg(feature = "mastodon-api")]
    {
        router = router.merge(handler::mastodon::routes());
    }

    let router = router
        .merge(graphql::routes(state.clone()))
        .fallback_service(get_service(
            ServeDir::new(frontend_dir).fallback(ServeFile::new(frontend_index_path)),
        ))
        // Even though this explicity has "prometheus" in the name, it just emits regular `metrics` calls
        .layer(PrometheusMetricLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    axum::Server::bind(&([0, 0, 0, 0], port).into())
        .serve(router.into_make_service())
        .await
        .unwrap();
}
