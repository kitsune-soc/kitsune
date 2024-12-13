use super::TokenResponse;
use crate::{
    error::{Error, Result},
    Client, ClientExtractor, OptionExt,
    params::ParamStorage,
};
use bytes::Bytes;
use headers::HeaderMapExt;
use std::future::Future;

pub trait Issuer {
    fn issue_token(
        &self,
        client: &Client<'_>,
        refresh_token: &str,
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
