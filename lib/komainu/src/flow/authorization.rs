use crate::{
    extract::ClientCredentials,
    flow::{self, OptionExt, TokenResponse},
    params::ParamStorage,
    Authorization, ClientExtractor,
};
use bytes::Bytes;
use std::future::Future;

pub trait Issuer {
    fn load_authorization(
        &self,
        auth_code: &str,
    ) -> impl Future<Output = Result<Option<Authorization<'_>>, flow::Error>> + Send;

    fn issue_token(
        &self,
        authorization: &Authorization<'_>,
    ) -> impl Future<Output = Result<TokenResponse<'_>, flow::Error>> + Send;
}

#[instrument(skip_all)]
pub async fn perform<CE, I>(
    req: http::Request<Bytes>,
    client_extractor: CE,
    token_issuer: I,
) -> Result<http::Response<Bytes>, flow::Error>
where
    CE: ClientExtractor,
    I: Issuer,
{
    let body: ParamStorage<&str, &str> = crate::extract::body(&req)?;
    let client_credentials =
        ClientCredentials::extract(req.headers(), &body).or_invalid_request()?;

    let (client_id, client_secret) = (
        client_credentials.client_id(),
        client_credentials.client_secret(),
    );

    let grant_type = body.get("grant_type").or_invalid_request()?;
    let code = body.get("code").or_invalid_request()?;
    let redirect_uri = body.get("redirect_uri").or_invalid_request()?;

    if *grant_type != "authorization_code" {
        error!(?client_id, "grant_type is not authorization_code");
        return Err(flow::Error::UnsupportedGrantType);
    }

    let client = client_extractor
        .extract(client_id, Some(client_secret))
        .await?;

    if client.redirect_uri != *redirect_uri {
        error!(?client_id, "redirect uri doesn't match");
        return Err(flow::Error::InvalidClient);
    }

    let Some(authorization) = token_issuer.load_authorization(code).await? else {
        return Err(flow::Error::InvalidGrant);
    };

    // This check is constant time :3
    if client != authorization.client {
        return Err(flow::Error::UnauthorizedClient);
    }

    if let Some(ref pkce) = authorization.pkce_payload {
        let code_verifier = body.get("code_verifier").or_invalid_request()?;
        pkce.verify(code_verifier)?;
    }

    let token = token_issuer.issue_token(&authorization).await?;
    let body = sonic_rs::to_vec(&token).unwrap();

    debug!("token successfully issued. building response");

    let response = http::Response::builder()
        .status(http::StatusCode::OK)
        .body(body.into())
        .unwrap();

    Ok(response)
}
