use super::TokenResponse;
use crate::{
    error::{fallible, yield_error, Result},
    extractor::ClientCredentials,
    params::ParamStorage,
    Authorization, ClientExtractor, Error, OptionExt,
};
use bytes::Bytes;
use std::future::Future;

pub trait Issuer {
    fn load_authorization(
        &self,
        auth_code: &str,
    ) -> impl Future<Output = Result<Option<Authorization<'_>>>> + Send;

    fn issue_token(
        &self,
        authorization: &Authorization<'_>,
    ) -> impl Future<Output = Result<TokenResponse<'_>>> + Send;
}

#[instrument(skip_all)]
pub async fn perform<CE, I>(
    req: http::Request<Bytes>,
    client_extractor: CE,
    token_issuer: I,
) -> http::Response<Bytes>
where
    CE: ClientExtractor,
    I: Issuer,
{
    let body: ParamStorage<&str, &str> = fallible!(crate::extractor::body(&req));

    let client_credentials =
        fallible!(ClientCredentials::extract(req.headers(), &body).or_unauthorized());

    let (client_id, client_secret) = (
        client_credentials.client_id(),
        client_credentials.client_secret(),
    );

    let grant_type = fallible!(body.get("grant_type").or_missing_param());
    let code = fallible!(body.get("code").or_missing_param());
    let redirect_uri = fallible!(body.get("redirect_uri").or_missing_param());

    if *grant_type != "authorization_code" {
        error!(?client_id, "grant_type is not authorization_code");
        yield_error!(Error::Unauthorized);
    }

    let client = fallible!(
        client_extractor
            .extract(client_id, Some(client_secret))
            .await
    );

    if client.redirect_uri != *redirect_uri {
        error!(?client_id, "redirect uri doesn't match");
        yield_error!(Error::Unauthorized);
    }

    let maybe_authorization = fallible!(token_issuer.load_authorization(code).await);
    let authorization = fallible!(maybe_authorization.or_unauthorized());

    // This check is constant time :3
    if client != authorization.client {
        yield_error!(Error::Unauthorized);
    }

    if let Some(ref pkce) = authorization.pkce_payload {
        let code_verifier = fallible!(body.get("code_verifier").or_unauthorized());
        fallible!(pkce.verify(code_verifier));
    }

    let token = fallible!(token_issuer.issue_token(&authorization).await);
    let body = sonic_rs::to_vec(&token).unwrap();

    debug!("token successfully issued. building response");

    http::Response::builder()
        .status(http::StatusCode::OK)
        .body(body.into())
        .unwrap()
}
