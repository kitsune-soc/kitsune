use crate::{
    activitypub::fetcher::FetchOptions,
    error::{Error, Result},
    state::Zustand,
};
use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{FromRequest, OriginalUri},
    response::{IntoResponse, Response},
    RequestExt,
};
use http::{request::Parts, StatusCode};
use kitsune_db::entity::accounts;
use kitsune_http_signatures::{
    ring::signature::{UnparsedPublicKey, RSA_PKCS1_2048_8192_SHA256},
    HttpVerifier,
};
use kitsune_type::ap::Activity;
use rsa::pkcs8::{Document, SubjectPublicKeyInfo};

pub struct SignedActivity(pub accounts::Model, pub Activity);

#[async_trait]
impl FromRequest<Zustand, Body> for SignedActivity {
    type Rejection = Response;

    async fn from_request(
        mut req: http::Request<Body>,
        state: &Zustand,
    ) -> Result<Self, Self::Rejection> {
        // Axum will cut out the "/users" part of the router (due to the nesting)
        // That's why we get the original URI here (which includes the full path)
        let OriginalUri(original_uri) = req
            .extract_parts()
            .await
            .map_err(IntoResponse::into_response)?;

        let (mut parts, body) = req
            .with_limited_body()
            .expect("[Bug] Payload size of inbox not limited")
            .into_parts();
        parts.uri = original_uri;

        let activity: Activity = match hyper::body::to_bytes(body).await {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(Error::from)?,
            Err(err) => {
                debug!(error = %err, "Failed to buffer body");
                return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
        };

        let ap_id = activity.actor();
        let remote_user = state.fetcher.fetch_actor(ap_id.into()).await?;
        if !verify_signature(&parts, &remote_user).await? {
            // Refetch the user and try again
            // Maybe they rekeyed

            let opts = FetchOptions::builder().refetch(true).url(ap_id).build();
            let remote_user = state.fetcher.fetch_actor(opts).await?;

            if !verify_signature(&parts, &remote_user).await? {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            }
        }

        Ok(Self(remote_user, activity))
    }
}

async fn verify_signature(parts: &Parts, remote_user: &accounts::Model) -> Result<bool> {
    let (_tag, public_key) = Document::from_pem(&remote_user.public_key).map_err(Error::from)?;
    let public_key: SubjectPublicKeyInfo<'_> = public_key.decode_msg().map_err(Error::from)?;
    let public_key = UnparsedPublicKey::new(
        &RSA_PKCS1_2048_8192_SHA256,
        public_key.subject_public_key.to_vec(),
    );

    let is_valid = HttpVerifier::default()
        .verify(parts, |_key_id| async move {
            // TODO: Select from the database by key ID
            Ok(public_key)
        })
        .await
        .is_ok();

    Ok(is_valid)
}
