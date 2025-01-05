use crate::{
    extract::ClientCredentials,
    flow::{self, OptionExt, TokenResponse},
    Client, ClientExtractor,
};
use std::future::Future;

pub trait Issuer {
    fn issue_token(
        &self,
        client: &Client<'_>,
        refresh_token: &str,
    ) -> impl Future<Output = Result<TokenResponse<'_>, flow::Error>> + Send;
}

#[instrument(skip_all)]
pub async fn perform<'a, CE, I>(
    req: &'a crate::Request<'_>,
    client_extractor: &CE,
    token_issuer: &'a I,
) -> Result<TokenResponse<'a>, flow::Error>
where
    CE: ClientExtractor,
    I: Issuer,
{
    let client_credentials =
        ClientCredentials::extract(&req.headers, &req.body).or_invalid_request()?;

    let (client_id, client_secret) = (
        client_credentials.client_id(),
        client_credentials.client_secret(),
    );

    let grant_type = req.body.get("grant_type").or_invalid_request()?;
    let refresh_token = req.body.get("refresh_token").or_invalid_request()?;

    if *grant_type != "refresh_token" {
        debug!(?client_id, "grant_type is not refresh_token");
        return Err(flow::Error::UnsupportedGrantType);
    }

    let client = client_extractor
        .extract(client_id, Some(client_secret))
        .await?;

    token_issuer.issue_token(&client, refresh_token).await
}
