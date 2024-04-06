use crate::oauth2::OAuthEndpoint;
use axum::{debug_handler, extract::State};
use kitsune_error::{kitsune_error, Error, ErrorType, Result};
use oxide_auth::endpoint::QueryParameter;
use oxide_auth_async::endpoint::{
    access_token::AccessTokenFlow, client_credentials::ClientCredentialsFlow, refresh::RefreshFlow,
};
use oxide_auth_axum::{OAuthRequest, OAuthResponse};

#[debug_handler(state = crate::state::Zustand)]
#[utoipa::path(post, path = "/oauth/token")]
pub async fn post(
    State(oauth_endpoint): State<OAuthEndpoint>,
    oauth_req: OAuthRequest,
) -> Result<OAuthResponse> {
    let grant_type = oauth_req
        .body()
        .and_then(|body| body.unique_value("grant_type"))
        .ok_or_else(|| kitsune_error!(type = ErrorType::BadRequest(None), "missing grant type"))?;

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
        _ => Err(OAuth2Error::UnknownGrantType),
    }
    .map_err(Error::from)
}
