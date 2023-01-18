use crate::{
    consts::USER_AGENT,
    db::model::{account, user},
    error::{Error, Result},
};
use base64::{engine::general_purpose, Engine};
use futures_util::{stream::FuturesUnordered, Stream, StreamExt};
use http::{Request, Uri};
use kitsune_http_signatures::{
    ring::signature::RsaKeyPair, HttpSigner, PrivateKey, SignatureComponent,
};
use kitsune_type::ap::Activity;
use reqwest::Client;
use sha2::{Digest, Sha256};

/// Delivers ActivityPub activities
///
/// Does not need to be Arc wrapped for cheap cloning. It's inherently cheap to clone.
#[derive(Clone)]
pub struct Deliverer {
    client: Client,
}

impl Deliverer {
    /// Deliver the activity to an inbox
    ///
    /// # Panics
    ///
    /// - Panics in case the inbox URL isn't actually a valid URL
    pub async fn deliver(
        &self,
        inbox_url: &str,
        account: &account::Model,
        user: &user::Model,
        activity: &Activity,
    ) -> Result<()> {
        let body = serde_json::to_string(&activity)?;
        let body_digest = general_purpose::STANDARD.encode(Sha256::digest(body.as_bytes()));
        let digest_header = format!("sha-256={body_digest}");

        let mut request = self
            .client
            .get(inbox_url)
            .header("Digest", &digest_header)
            .body(body.clone())
            .build()?;

        let key_id = format!("{}#main-key", account.url);
        let headers = request.headers().clone();
        let uri =
            Uri::try_from(request.url().as_str()).expect("[Bug] Invalid URI stored in url::Url");
        let method = request.method().clone();

        let mut dummy_request = Request::builder().uri(uri).method(method);
        *dummy_request.headers_mut().unwrap() = headers;
        let dummy_request = dummy_request.body(()).unwrap();
        let private_key = PrivateKey::builder()
            .key_id(&key_id)
            .key(RsaKeyPair::from_pkcs8(user.private_key.as_bytes())?)
            .build()
            .unwrap();

        let (parts, _body) = dummy_request.into_parts();
        let (header_name, header_value) = HttpSigner::builder()
            .parts(&parts)
            .build()
            .unwrap()
            .sign(
                private_key,
                vec![
                    SignatureComponent::RequestTarget,
                    SignatureComponent::Header("digest"),
                    SignatureComponent::Header("date"),
                ],
            )
            .await?;

        request.headers_mut().insert(header_name, header_value);
        if !self.client.execute(request).await?.status().is_success() {
            todo!("return error");
        }

        Ok(())
    }

    pub async fn deliver_many<S, E>(
        &self,
        account: &account::Model,
        user: &user::Model,
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

impl Default for Deliverer {
    fn default() -> Self {
        Self {
            client: Client::builder().user_agent(USER_AGENT).build().unwrap(),
        }
    }
}
