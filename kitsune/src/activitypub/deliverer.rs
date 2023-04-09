use crate::{
    consts::USER_AGENT,
    error::{Error, Result},
    service::federation_filter::FederationFilterService,
};
use autometrics::autometrics;
use base64::{engine::general_purpose, Engine};
use futures_util::{stream::FuturesUnordered, Stream, StreamExt};
use http::{Method, Request};
use kitsune_db::entity::{accounts, users};
use kitsune_http_client::Client;
use kitsune_http_signatures::{ring::signature::RsaKeyPair, PrivateKey};
use kitsune_type::ap::Activity;
use rsa::pkcs8::SecretDocument;
use serde::Serialize;
use sha2::{Digest, Sha256};
use typed_builder::TypedBuilder;
use url::Url;

/// Delivers ActivityPub activities
///
/// Does not need to be Arc wrapped for cheap cloning. It's inherently cheap to clone.
#[derive(Clone, TypedBuilder)]
pub struct Deliverer {
    #[builder(default = Client::builder().user_agent(USER_AGENT).unwrap().build())]
    client: Client,
    federation_filter: FederationFilterService,
}

impl Deliverer {
    /// Deliver the activity to an inbox
    ///
    /// # Panics
    ///
    /// - Panics in case the inbox URL isn't actually a valid URL
    #[instrument(skip_all, fields(%inbox_url, activity_url = %activity.rest.id))]
    #[autometrics(track_concurrency)]
    pub async fn deliver<T>(
        &self,
        inbox_url: &str,
        account: &accounts::Model,
        user: &users::Model,
        activity: &Activity<T>,
    ) -> Result<()>
    where
        T: Serialize,
    {
        if !self
            .federation_filter
            .is_url_allowed(&Url::parse(inbox_url)?)?
        {
            return Ok(());
        }

        let body = serde_json::to_string(&activity)?;
        let body_digest = general_purpose::STANDARD.encode(Sha256::digest(body.as_bytes()));
        let digest_header = format!("sha-256={body_digest}");

        let request = Request::builder()
            .method(Method::POST)
            .uri(inbox_url)
            .header("Digest", digest_header)
            .body(body.into())?;

        let key_id = format!("{}#main-key", account.url);
        let (_tag, pkcs8_document) = SecretDocument::from_pem(&user.private_key)?;
        let private_key = PrivateKey::builder()
            .key_id(&key_id)
            .key(RsaKeyPair::from_pkcs8(pkcs8_document.as_bytes())?)
            .build()
            .unwrap();

        let response = self.client.execute_signed(request, private_key).await?;
        debug!(status_code = %response.status(), "successfully executed http request");
        if !response.status().is_success() {
            let status_code = response.status();
            let body = response.text().await?;
            error!(%status_code, %body, %inbox_url, "failed to deliver activity");
        }

        Ok(())
    }

    pub async fn deliver_many<S, E>(
        &self,
        account: &accounts::Model,
        user: &users::Model,
        activity: &Activity,
        inbox_stream: S,
    ) -> Result<()>
    where
        S: Stream<Item = Result<Vec<String>, E>>,
        Error: From<E>,
    {
        tokio::pin!(inbox_stream);

        while let Some(inbox_chunk) = inbox_stream.next().await.transpose()? {
            let mut concurrent_resolver: FuturesUnordered<_> = inbox_chunk
                .iter()
                .map(|inbox| self.deliver(inbox, account, user, activity))
                .collect();

            while let Some(delivery_result) = concurrent_resolver.next().await {
                if let Err(err) = delivery_result {
                    error!(error = %err, "Failed to deliver activity to inbox");
                }
            }
        }

        Ok(())
    }
}
