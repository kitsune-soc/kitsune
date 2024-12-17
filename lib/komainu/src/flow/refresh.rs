use super::TokenResponse;
use crate::{
    error::{Error, Result},
    extract::ClientCredentials,
    params::ParamStorage,
    Client, ClientExtractor,
};
use bytes::Bytes;
use std::future::Future;

pub trait Issuer {
    fn issue_token(
        &self,
        client: &Client<'_>,
        refresh_token: &str,
    ) -> impl Future<Output = Result<TokenResponse<'_>>> + Send;
}

#[instrument(skip_all)]
pub async fn perform<CE, I>(
    req: http::Request<Bytes>,
    client_extractor: CE,
    token_issuer: I,
) -> Result<http::Response<Bytes>>
where
    CE: ClientExtractor,
    I: Issuer,
{
    let body: ParamStorage<&str, &str> = crate::extract::body(&req)?;
    let client_credentials = ClientCredentials::extract(req.headers(), &body).or_unauthorized()?;

    let (client_id, client_secret) = (
        client_credentials.client_id(),
        client_credentials.client_secret(),
    );

    let grant_type = body.get("grant_type").or_missing_param()?;
    let refresh_token = body.get("refresh_token").or_missing_param()?;

    if *grant_type != "refresh_token" {
        debug!(?client_id, "grant_type is not refresh_token");
        return Err(Error::Unauthorized);
    }

    let client = client_extractor
        .extract(client_id, Some(client_secret))
        .await?;

    let token = token_issuer.issue_token(&client, refresh_token).await?;
    let body = sonic_rs::to_vec(&token).unwrap();

    debug!("token successfully issued. building response");

    let response = http::Response::builder()
        .status(http::StatusCode::OK)
        .body(body.into())
        .unwrap();

    Ok(response)
}
