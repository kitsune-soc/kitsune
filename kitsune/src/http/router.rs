use super::{handler, middleware, serve_frontend, trace_layer, X_REQUEST_ID};
use crate::state::Zustand;
use axum::{routing, Router};
use color_eyre::eyre::{self, Context};
use cursiv::CsrfLayer;
use flashy::FlashLayer;
use kitsune_config::server;
use std::time::Duration;
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
};
use tower_http_digest::VerifyDigestLayer;
use tower_stop_using_brave::StopUsingBraveLayer;
use tower_x_clacks_overhead::XClacksOverheadLayer;

#[allow(clippy::too_many_lines)]
pub fn create(state: Zustand, server_config: &server::Configuration) -> eyre::Result<Router> {
    let router = Router::new()
        .route(
            "/confirm-account/{confirmation_token}",
            routing::get(handler::confirm_account::get),
        )
        .route("/emojis/{id}", routing::get(handler::custom_emojis::get))
        .route("/media/{id}", routing::get(handler::media::get))
        .route(
            "/nodeinfo/2.1",
            routing::get(handler::nodeinfo::two_one::get),
        )
        .nest(
            "/oauth",
            Router::new()
                .route(
                    "/authorize",
                    routing::get(handler::oauth::authorize::get)
                        .post(handler::oauth::authorize::post),
                )
                .route("/token", routing::post(handler::oauth::token::post))
                .layer(axum::middleware::from_fn(middleware::json_to_urlencoded)),
        )
        .nest(
            "/posts",
            Router::new()
                .route("/{id}", routing::get(handler::posts::get))
                .route(
                    "/{id}/activity",
                    routing::get(handler::posts::activity::get),
                ),
        )
        .nest(
            "/users",
            Router::new()
                .route("/{user_id}", routing::get(handler::users::get))
                .route(
                    "/{user_id}/followers",
                    routing::get(handler::users::followers::get),
                )
                .route(
                    "/{user_id}/following",
                    routing::get(handler::users::following::get),
                )
                .route(
                    "/{user_id}/inbox",
                    routing::post(handler::users::inbox::post).layer(VerifyDigestLayer::default()),
                )
                .route(
                    "/{user_id}/outbox",
                    routing::get(handler::users::outbox::get),
                ),
        )
        .nest(
            "/.well-known",
            Router::new()
                .route(
                    "/nodeinfo",
                    routing::get(handler::well_known::nodeinfo::get),
                )
                .route(
                    "/webfinger",
                    routing::get(handler::well_known::webfinger::get),
                ),
        )
        .route("/public/{*path}", routing::get(handler::public::get));

    #[cfg(feature = "oidc")]
    let router = router.route("/oidc/callback", routing::get(handler::oidc::callback::get));

    #[cfg(feature = "graphql-api")]
    let router = {
        use super::graphql;
        use async_graphql_axum::GraphQLSubscription;
        use axum::Extension;

        let schema = graphql::schema(state.clone());

        router.merge(
            Router::new()
                .nest(
                    "/graphql",
                    Router::new()
                        .route("/", routing::any(graphql::graphql))
                        .route("/explorer", routing::get(graphql::explorer))
                        .route_service("/ws", GraphQLSubscription::new(schema.clone())),
                )
                .layer(Extension(schema)),
        )
    };

    #[cfg(feature = "mastodon-api")]
    let router = {
        use axum::extract::DefaultBodyLimit;

        router.nest(
            "/api",
            Router::new()
                .nest(
                    "/v1",
                    Router::new()
                        .nest(
                            "/accounts",
                            Router::new()
                                .route(
                                    "/{id}",
                                    routing::get(handler::mastodon::api::v1::accounts::get),
                                )
                                .route(
                                    "/{id}/follow",
                                    routing::post(handler::mastodon::api::v1::accounts::follow::post),
                                )
                                .route(
                                    "/{id}/statuses",
                                    routing::get(handler::mastodon::api::v1::accounts::statuses::get),
                                )
                                .route(
                                    "/{id}/unfollow",
                                    routing::post(handler::mastodon::api::v1::accounts::unfollow::post),
                                )
                                .route(
                                    "/lookup",
                                    routing::get(handler::mastodon::api::v1::accounts::lookup::get),
                                )
                                .route(
                                    "/relationships",
                                    routing::get(
                                        handler::mastodon::api::v1::accounts::relationships::get,
                                    ),
                                )
                                .route(
                                    "/update_credentials",
                                    routing::patch(
                                        handler::mastodon::api::v1::accounts::update_credentials::patch,
                                    ),
                                )
                                .route(
                                    "/verify_credentials",
                                    routing::get(
                                        handler::mastodon::api::v1::accounts::verify_credentials::get,
                                    ),
                                ),
                        )
                        .route(
                            "/apps",
                            routing::post(handler::mastodon::api::v1::apps::post),
                        )
                        .route(
                            "/custom_emojis",
                            routing::get(handler::mastodon::api::v1::custom_emojis::get),
                        )
                        .nest(
                            "/follow_requests",
                            Router::new()
                                .route(
                                    "/",
                                    routing::get(handler::mastodon::api::v1::follow_requests::get),
                                )
                                .route(
                                    "/{id}/authorize",
                                    routing::post(
                                        handler::mastodon::api::v1::follow_requests::accept::post,
                                    ),
                                )
                                .route(
                                    "/{id}/reject",
                                    routing::post(
                                        handler::mastodon::api::v1::follow_requests::reject::post,
                                    ),
                                ),
                        )
                        .route(
                            "/instance",
                            routing::get(handler::mastodon::api::v1::instance::get),
                        )
                        .nest(
                            "/media",
                            Router::new()
                                .route(
                                    "/",
                                    routing::post(handler::mastodon::api::v1::media::post).layer(
                                        DefaultBodyLimit::max(
                                            server_config.max_upload_size.to_bytes() as usize
                                        ),
                                    ),
                                )
                                .route(
                                    "/{id}",
                                    routing::get(handler::mastodon::api::v1::media::get)
                                        .put(handler::mastodon::api::v1::media::put),
                                ),
                        )
                        .nest(
                            "/notifications",
                            Router::new()
                                .route(
                                    "/",
                                    routing::get(handler::mastodon::api::v1::notifications::get),
                                )
                                .route(
                                    "/{id}",
                                    routing::get(handler::mastodon::api::v1::notifications::get_by_id),
                                )
                                .route(
                                    "/{id}/dismiss",
                                    routing::post(
                                        handler::mastodon::api::v1::notifications::dismiss::post,
                                    ),
                                )
                                .route(
                                    "/clear",
                                    routing::post(
                                        handler::mastodon::api::v1::notifications::clear::post,
                                    ),
                                ),
                        )
                        .nest(
                            "/statuses",
                            Router::new()
                                .route(
                                    "/",
                                    routing::post(handler::mastodon::api::v1::statuses::post),
                                )
                                .route(
                                    "/{id}",
                                    routing::delete(handler::mastodon::api::v1::statuses::delete)
                                        .get(handler::mastodon::api::v1::statuses::get)
                                        .put(handler::mastodon::api::v1::statuses::put),
                                )
                                .route(
                                    "/{id}/context",
                                    routing::get(handler::mastodon::api::v1::statuses::context::get),
                                )
                                .route(
                                    "/{id}/favourite",
                                    routing::post(
                                        handler::mastodon::api::v1::statuses::favourite::post,
                                    ),
                                )
                                .route(
                                    "/{id}/favourited_by",
                                    routing::get(
                                        handler::mastodon::api::v1::statuses::favourited_by::get,
                                    ),
                                )
                                .route(
                                    "/{id}/reblog",
                                    routing::post(handler::mastodon::api::v1::statuses::reblog::post),
                                )
                                .route(
                                    "/{id}/reblogged_by",
                                    routing::get(
                                        handler::mastodon::api::v1::statuses::reblogged_by::get,
                                    ),
                                )
                                .route(
                                    "/{id}/source",
                                    routing::get(handler::mastodon::api::v1::statuses::source::get),
                                )
                                .route(
                                    "/{id}/unfavourite",
                                    routing::post(
                                        handler::mastodon::api::v1::statuses::unfavourite::post,
                                    ),
                                )
                                .route(
                                    "/{id}/unreblog",
                                    routing::post(handler::mastodon::api::v1::statuses::unreblog::post),
                                ),
                        )
                        .nest(
                            "/timelines",
                            Router::new()
                                .route(
                                    "/home",
                                    routing::get(handler::mastodon::api::v1::timelines::home::get),
                                )
                                .route(
                                    "/public",
                                    routing::get(handler::mastodon::api::v1::timelines::public::get),
                                ),
                        ),
                )
                .nest(
                    "/v2",
                    Router::new()
                        .nest(
                            "/media",
                            Router::new()
                                .route(
                                    "/",
                                    routing::post(handler::mastodon::api::v1::media::post).layer(
                                        DefaultBodyLimit::max(
                                            server_config.max_upload_size.to_bytes() as usize
                                        ),
                                    ),
                                )
                                .route(
                                    "/{id}",
                                    routing::get(handler::mastodon::api::v1::media::get)
                                        .put(handler::mastodon::api::v1::media::put),
                                ),
                        )
                        .route(
                            "/search",
                            routing::get(handler::mastodon::api::v2::search::get),
                        ),
                ),
        )
    };

    let mut router = router.fallback_service(serve_frontend(server_config));

    if !server_config.clacks_overhead.is_empty() {
        let clacks_overhead_layer =
            XClacksOverheadLayer::new(server_config.clacks_overhead.iter().map(AsRef::as_ref))
                .wrap_err("Invalid clacks overhead values")?;

        router = router.layer(clacks_overhead_layer);
    }

    if server_config.deny_brave_browsers {
        router = router.layer(StopUsingBraveLayer::default());
    }

    let router = router
        .layer(CatchPanicLayer::new())
        .layer(CorsLayer::permissive())
        .layer(CsrfLayer::generate()) // TODO: Make this configurable instead of random
        .layer(FlashLayer::generate()) // TODO: Make this configurable instead of random
        .layer(TimeoutLayer::new(Duration::from_secs(
            server_config.request_timeout_secs,
        )))
        .layer(trace_layer())
        .layer(PropagateRequestIdLayer::new(X_REQUEST_ID.clone()))
        .layer(SetRequestIdLayer::new(
            X_REQUEST_ID.clone(),
            MakeRequestUuid,
        ))
        .with_state(state);

    Ok(router)
}
