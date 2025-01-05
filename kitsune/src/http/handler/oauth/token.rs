use crate::oauth2::{AuthIssuer, ClientExtractor, RefreshIssuer};
use axum::{debug_handler, extract::State};
use kitsune_error::Result;

#[debug_handler(state = crate::state::Zustand)]
pub async fn post(
    State(db_pool): State<kitsune_db::PgPool>,
    request: axum::extract::Request,
) -> Result<axum::response::Response> {
    let oauth_req = komainu::Request::read_from(request).await?;

    let impls = komainu::flow::Impls {
        auth_issuer: AuthIssuer::builder().db_pool(db_pool.clone()).build(),
        client_extractor: ClientExtractor::builder().db_pool(db_pool.clone()).build(),
        refresh_issuer: RefreshIssuer::builder().db_pool(db_pool).build(),
    };
    let response = {
        let oauth_response = komainu::flow::dispatch(&oauth_req, &impls).await?;
        oauth_response.map(axum::body::Body::from)
    };

    Ok(response)
}
