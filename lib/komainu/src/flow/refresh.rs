use super::TokenResponse;
use crate::{
    error::{fallible, yield_error, Error, Result},
    extractor::ClientCredentials,
    params::ParamStorage,
    Client, ClientExtractor, OptionExt,
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
    let refresh_token = fallible!(body.get("refresh_token").or_missing_param());

    if *grant_type != "refresh_token" {
        debug!(?client_id, "grant_type is not refresh_token");
        yield_error!(Error::Unauthorized);
    }

    let client = fallible!(
        client_extractor
            .extract(
                client_credentials.client_id(),
                Some(client.credentials.client_secret())
            )
            .await
    );

    let token = fallible!(token_issuer.issue_token(&client, refresh_token).await);
    let body = sonic_rs::to_vec(&token).unwrap();

    debug!("token successfully issued. building response");

    http::Response::builder()
        .status(http::StatusCode::OK)
        .body(body.into())
        .unwrap()
}
