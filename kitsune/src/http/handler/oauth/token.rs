use crate::http::extractor::Json;
use axum::{debug_handler, extract::State};
use kitsune_error::{kitsune_error, Error, ErrorType, Result};

#[debug_handler(state = crate::state::Zustand)]
pub async fn post(
    request: axum::extract::Request,
) -> Result<Json<komainu::flow::TokenResponse<'static>>> {
    let grant_type = oauth_req
        .body()
        .and_then(|body| body.unique_value("grant_type"))
        .ok_or_else(|| kitsune_error!(type = ErrorType::BadRequest, "missing grant type"))?;

    match grant_type.as_ref() {
        "authorization_code" => {
            let mut flow = AccessTokenFlow::prepare(oauth_endpoint)?;
            flow.allow_credentials_in_body(true);
            AccessTokenFlow::execute(&mut flow, oauth_req).await
        }
        "client_credentials" => {
            let mut flow = ClientCredentialsFlow::prepare(oauth_endpoint)?;
            flow.allow_credentials_in_body(true);
            ClientCredentialsFlow::execute(&mut flow, oauth_req).await
        }
        "refresh_token" => {
            let mut flow = RefreshFlow::prepare(oauth_endpoint)?;
            RefreshFlow::execute(&mut flow, oauth_req).await
        }
        _ => Err(kitsune_error!(
            type = ErrorType::BadRequest.with_body("unknown grant type"),
            format!("unknown grant type: {grant_type}")
        )),
    }
    .map_err(Error::from)
}
