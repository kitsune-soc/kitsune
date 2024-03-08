use crate::error::{Error, Result};
use autometrics::autometrics;
use futures_util::{stream::FuturesUnordered, Stream, StreamExt};
use http::{Method, Request};
use kitsune_core::consts::USER_AGENT;
use kitsune_db::model::{account::Account, user::User};
use kitsune_federation_filter::FederationFilter;
use kitsune_http_client::Client;
use kitsune_type::ap::Activity;
use kitsune_wasm_mrf::{MrfService, Outcome};
use sha2::{Digest, Sha256};
use std::pin::pin;
use typed_builder::TypedBuilder;
use url::Url;

/// Delivers ActivityPub activities
///
/// Does not need to be Arc wrapped for cheap cloning. It's inherently cheap to clone.
#[derive(Clone, TypedBuilder)]
pub struct Deliverer {
    #[builder(default = Client::builder().user_agent(USER_AGENT).unwrap().build())]
    client: Client,
    federation_filter: FederationFilter,
    mrf_service: MrfService,
}

impl Deliverer {
    /// Deliver the activity to an inbox
    #[autometrics(track_concurrency)]
    #[instrument(skip_all, fields(%inbox_url, activity_url = %activity.id))]
    pub async fn deliver(
        &self,
        inbox_url: &str,
        account: &Account,
        user: &User,
        activity: &Activity,
    ) -> Result<()> {
        if !self
            .federation_filter
            .is_url_allowed(&Url::parse(inbox_url)?)?
        {
            return Ok(());
        }

        let body = match self.mrf_service.handle_outgoing(activity).await? {
            Outcome::Accept(body) => body,
            Outcome::Reject => todo!(),
        };

        let body_digest = base64_simd::STANDARD.encode_to_string(Sha256::digest(body.as_bytes()));
        let digest_header = format!("sha-256={body_digest}");

        let request = Request::builder()
            .method(Method::POST)
            .uri(inbox_url)
            .header("Digest", digest_header)
            .body(body.into())?;

        let response = self
            .client
            .execute_signed(request, &account.public_key_id, &user.private_key)
            .await?;

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
        account: &Account,
        user: &User,
        activity: &Activity,
        inbox_stream: S,
    ) -> Result<()>
    where
        S: Stream<Item = Result<Vec<String>, E>>,
        Error: From<E>,
    {
        let mut inbox_stream = pin!(inbox_stream);

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
