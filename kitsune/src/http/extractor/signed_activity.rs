use crate::{
    error::{Error, Result},
    state::AppState,
};
use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{FromRequest, OriginalUri},
    response::{IntoResponse, Response},
    RequestExt,
};
use bytes::Buf;
use const_oid::db::rfc8410::ID_ED_25519;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use http::{request::Parts, StatusCode};
use kitsune_core::{activitypub::fetcher::FetchOptions, error::ApiError};
use kitsune_db::{model::account::Account, schema::accounts, PgPool};
use kitsune_http_signatures::{
    ring::signature::{
        UnparsedPublicKey, VerificationAlgorithm, ED25519, RSA_PKCS1_2048_8192_SHA256,
    },
    HttpVerifier,
};
use kitsune_type::ap::Activity;
use rsa::pkcs8::{Document, SubjectPublicKeyInfoRef};
use scoped_futures::ScopedFutureExt;

/// Parses the body into an ActivityPub activity and verifies the HTTP signature
///
/// This extractor ensures that the activity belongs to the person that signed this activity
/// but not that the activity matches the object, so beware of that.
pub struct SignedActivity(pub Account, pub Activity);

#[async_trait]
impl FromRequest<AppState, Body> for SignedActivity {
    type Rejection = Response;

    async fn from_request(
        mut req: http::Request<Body>,
        state: &AppState,
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

        let activity: Activity = match hyper::body::aggregate(body).await {
            Ok(body) => simd_json::from_reader(body.reader()).map_err(Error::from)?,
            Err(error) => {
                debug!(?error, "Failed to buffer body");
                return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
        };

        let ap_id = activity.actor();
        let remote_user = state
            .fetcher
            .fetch_actor(ap_id.into())
            .await
            .map_err(Error::from)?;

        if !verify_signature(&parts, &state.db_pool, Some(&remote_user)).await? {
            // Refetch the user and try again. Maybe they rekeyed
            let opts = FetchOptions::builder().refetch(true).url(ap_id).build();
            let remote_user = state.fetcher.fetch_actor(opts).await.map_err(Error::from)?;

            if !verify_signature(&parts, &state.db_pool, Some(&remote_user)).await? {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            }
        }

        Ok(Self(remote_user, activity))
    }
}

async fn verify_signature(
    parts: &Parts,
    db_conn: &PgPool,
    expected_account: Option<&Account>,
) -> Result<bool> {
    let is_valid = HttpVerifier::default()
        .verify(parts, |key_id| async move {
            let remote_user: Account = db_conn
                .with_connection(|db_conn| {
                    accounts::table
                        .filter(accounts::public_key_id.eq(key_id))
                        .select(Account::as_select())
                        .first(db_conn)
                        .scoped()
                })
                .await?;

            // If we have an expected account, which we have in the case of an incoming new activity,
            // then we do this comparison.
            // In the case of incoming activities, this is to ensure that the account this will be attributed to is actually the one signing it.
            // Otherwise a random person with a key that's known to the database could start signing activities willy-nilly and the server would accept it.
            if let Some(expected_account) = expected_account {
                if expected_account.url != remote_user.url {
                    return Err(ApiError::Unauthorised.into());
                }
            }

            let (_tag, public_key) =
                Document::from_pem(&remote_user.public_key).map_err(Error::from)?;
            let public_key: SubjectPublicKeyInfoRef<'_> =
                public_key.decode_msg().map_err(Error::from)?;
            let public_key_bytes = public_key.subject_public_key.raw_bytes().to_vec();

            // TODO: Replace this with an actual comparison as soon as the new const_oid version is out
            let algorithm: &'static dyn VerificationAlgorithm =
                if public_key.algorithm.oid.as_bytes() == ID_ED_25519.as_bytes() {
                    &ED25519
                } else {
                    &RSA_PKCS1_2048_8192_SHA256
                };

            Ok(UnparsedPublicKey::new(algorithm, public_key_bytes))
        })
        .await
        .is_ok();

    Ok(is_valid)
}
