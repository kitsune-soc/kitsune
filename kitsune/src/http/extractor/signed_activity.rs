use crate::{
    activitypub::fetcher::FetchOptions,
    error::{ApiError, Error, Result},
    state::Zustand,
};
use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{FromRequest, OriginalUri},
    response::{IntoResponse, Response},
    RequestExt,
};
use const_oid::db::rfc8410::ID_ED_25519;
use http::{request::Parts, StatusCode};
use kitsune_db::entity::{accounts, prelude::Accounts};
use kitsune_http_signatures::{
    ring::signature::{UnparsedPublicKey, ED25519, RSA_PKCS1_2048_8192_SHA256},
    HttpVerifier,
};
use kitsune_type::ap::Activity;
use rsa::pkcs8::{Document, SubjectPublicKeyInfo};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

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
        if !verify_signature(&parts, &state.db_conn).await? {
            // Refetch the user and try again. Maybe they rekeyed
            let opts = FetchOptions::builder().refetch(true).url(ap_id).build();
            state.fetcher.fetch_actor(opts).await?;

            if !verify_signature(&parts, &state.db_conn).await? {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            }
        }

        Ok(Self(remote_user, activity))
    }
}

async fn verify_signature(parts: &Parts, db_conn: &DatabaseConnection) -> Result<bool> {
    let is_valid = HttpVerifier::default()
        .verify(parts, |key_id| async move {
            let remote_user = Accounts::find()
                .filter(accounts::Column::PublicKeyId.eq(key_id))
                .one(db_conn)
                .await?
                .ok_or(ApiError::NotFound)?;

            let (_tag, public_key) =
                Document::from_pem(&remote_user.public_key).map_err(Error::from)?;
            let public_key: SubjectPublicKeyInfo<'_> =
                public_key.decode_msg().map_err(Error::from)?;

            // TODO: Replace this with an actual comparison as soon as the new const_oid version is out
            let public_key = if public_key.algorithm.oid.as_bytes() == ID_ED_25519.as_bytes() {
                UnparsedPublicKey::new(&ED25519, public_key.subject_public_key.to_vec())
            } else {
                UnparsedPublicKey::new(
                    &RSA_PKCS1_2048_8192_SHA256,
                    public_key.subject_public_key.to_vec(),
                )
            };

            Ok(public_key)
        })
        .await
        .is_ok();

    Ok(is_valid)
}
