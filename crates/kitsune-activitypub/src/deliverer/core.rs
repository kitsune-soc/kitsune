use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::{Stream, StreamExt, stream::FuturesUnordered};
use http::{Method, Request};
use kitsune_db::{
    PgPool,
    model::{Account, User},
    schema::{accounts_activitypub, cryptographic_keys},
    with_connection,
};
use kitsune_error::{Error, Result};
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
    http_client: Client,
    federation_filter: FederationFilter,
    mrf_service: MrfService,
    db_pool: PgPool,
}

impl Deliverer {
    /// Deliver the activity to an inbox
    #[cfg_attr(not(coverage), instrument(skip_all, fields(%inbox_url, activity_url = %activity.id)))]
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

        let (key_id, private_key) = with_connection!(self.db_pool, |db_conn| {
            accounts_activitypub::table
                .filter(accounts_activitypub::account_id.eq(account.id))
                .inner_join(
                    cryptographic_keys::table
                        .on(accounts_activitypub::key_id.eq(cryptographic_keys::key_id)),
                )
                .select((
                    accounts_activitypub::key_id,
                    cryptographic_keys::private_key_der,
                ))
                .first::<(String, Option<Vec<u8>>)>(db_conn)
                .await
        })?;

        let private_key =
            private_key.ok_or_else(|| Error::msg("Private key not found for account"))?;

        let response = self
            .http_client
            .execute_signed(request, &key_id, &private_key)
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
