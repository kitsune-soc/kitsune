use super::TokenResponse;
use crate::{error::Result, params::ParamStorage, Client, ClientExtractor, Error, OptionExt};
use bytes::Bytes;
use headers::HeaderMapExt;
use std::future::Future;

pub trait Issuer {
    fn issue_token(
        &self,
        client: &Client<'_>,
        auth_code: &str,
    ) -> impl Future<Output = Result<TokenResponse<'_>>>;
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
    let body: ParamStorage<&str, &str> = crate::deserialize_body(&req)?;

    let basic_auth = req
        .headers()
        .typed_get::<headers::Authorization<headers::authorization::Basic>>();

    let (client_id, client_secret) = if let Some(ref auth) = basic_auth {
        (auth.username(), auth.password())
    } else {
        debug!("attempting to read client credentials from body (naughty :3)");

        // As a fallback, try to read from the body.
        // Not recommended but some clients do this. Done to increase compatibility.
        let client_id = body.get("client_id").or_missing_param()?;
        let client_secret = body.get("client_secret").or_missing_param()?;

        (*client_id, *client_secret)
    };

    let grant_type = body.get("grant_type").or_missing_param()?;
    let code = body.get("code").or_missing_param()?;
    let redirect_uri = body.get("redirect_uri").or_missing_param()?;

    if *grant_type != "authorization_code" {
        error!(?client_id, "grant_type is not authorization_code");
        return Err(Error::Unauthorized);
    }

    let client = client_extractor
        .extract(client_id, Some(client_secret))
        .await?;

    if client.redirect_uri != *redirect_uri {
        error!(?client_id, "redirect uri doesn't match");
        return Err(Error::Unauthorized);
    }

    let token = token_issuer.issue_token(&client, code).await?;
    let body = sonic_rs::to_vec(&token).unwrap();

    debug!("token successfully issued. building response");

    let response = http::Response::builder()
        .status(http::StatusCode::OK)
        .body(body.into())
        .unwrap();

    Ok(response)
}
