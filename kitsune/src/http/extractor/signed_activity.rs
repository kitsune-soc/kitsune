use crate::{
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
use bytes::Buf;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use http::StatusCode;
use http_body_util::BodyExt;
use kitsune_core::{error::HttpError, traits::fetcher::AccountFetchOptions};
use kitsune_db::{model::account::Account, schema::accounts, PgPool};
use kitsune_type::ap::Activity;
use kitsune_wasm_mrf::Outcome;
use scoped_futures::ScopedFutureExt;

/// Parses the body into an ActivityPub activity and verifies the HTTP signature
///
/// This extractor ensures that the activity belongs to the person that signed this activity
/// but not that the activity matches the object, so beware of that.
pub struct SignedActivity(pub Account, pub Activity);

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

        let (mut parts, body) = req.with_limited_body().into_parts();
        parts.uri = original_uri;

        let aggregated_body = match body.collect().await {
            Ok(body) => body.to_bytes(),
            Err(error) => {
                debug!(?error, "Failed to buffer body");
                return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
        };

        let activity: Activity =
            simd_json::from_reader(aggregated_body.clone().reader()).map_err(Error::from)?;
        let Ok(str_body) = simdutf8::basic::from_utf8(&aggregated_body) else {
            debug!("Malformed body. Not UTF-8");
            return Err(StatusCode::BAD_REQUEST.into_response());
        };

        let Outcome::Accept(str_body) = state
            .service
            .mrf
            .handle_incoming(activity.r#type.as_ref(), str_body)
            .await
            .map_err(Error::from)?
        else {
            debug!("sending rejection");
            return Err(StatusCode::BAD_REQUEST.into_response());
        };

        let activity: Activity = match simd_json::from_reader(str_body.as_ref().as_bytes()) {
            Ok(activity) => activity,
            Err(error) => {
                debug!(?error, "Malformed activity");
                return Err(StatusCode::BAD_REQUEST.into_response());
            }
        };

        let ap_id = activity.actor.as_str();
        let Some(remote_user) = state
            .fetcher
            .fetch_account(ap_id.into())
            .await
            .map_err(Error::Fetcher)?
        else {
            return Err(Error::CoreHttp(HttpError::BadRequest).into());
        };

        let request = http::Request::from_parts(parts, ());
        if !verify_signature(&request, &state.db_pool, Some(&remote_user)).await? {
            // Refetch the user and try again. Maybe they rekeyed
            let opts = AccountFetchOptions::builder()
                .refetch(true)
                .url(ap_id)
                .build();

            let Some(remote_user) = state
                .fetcher
                .fetch_account(opts)
                .await
                .map_err(Error::Fetcher)?
            else {
                return Err(Error::CoreHttp(HttpError::BadRequest).into());
            };

            if !verify_signature(&request, &state.db_pool, Some(&remote_user)).await? {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            }
        }

        Ok(Self(remote_user, activity))
    }
}

async fn verify_signature(
    req: &http::Request<()>,
    db_conn: &PgPool,
    expected_account: Option<&Account>,
) -> Result<bool> {
    let is_valid = http_signatures::cavage::easy::verify(req, |key_id| {
        async move {
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
                    return Err(HttpError::Unauthorised.into());
                }
            }

            Ok::<_, Error>(remote_user.public_key)
        }
        .scoped()
    })
    .await
    .is_ok();

    Ok(is_valid)
}
