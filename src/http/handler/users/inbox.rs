use crate::{error::Error, state::State};
use async_trait::async_trait;
use axum::{
    body::Body,
    debug_handler,
    extract::{FromRequest, RequestParts},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use phenomenon_ap::Activity;
use phenomenon_http_signatures::Request;
use rsa::pkcs1::EncodeRsaPublicKey;

pub struct SignedActivity(pub Activity);

#[async_trait]
impl FromRequest<Body> for SignedActivity {
    type Rejection = Response;

    async fn from_request(parts: &mut RequestParts<Body>) -> Result<Self, Self::Rejection> {
        let (Extension(state), Json(activity)) =
            <(Extension<State>, Json<Activity>)>::from_request(parts)
                .await
                .map_err(IntoResponse::into_response)?;

        let Some(ap_id) = activity.rest.attributed_to() else {
            return Err(StatusCode::BAD_REQUEST.into_response());
        };
        let remote_user = state.fetcher.fetch_actor(ap_id).await?;
        let Some(public_key) = remote_user.public_key()? else {
            error!(user_id = %remote_user.id, "Missing RSA public key");
            return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        };

        let headers = parts.headers().clone();
        let method = parts.method().clone();
        let uri = parts.uri().clone();
        let is_valid = crate::blocking::cpu(move || {
            let request = Request {
                headers: &headers,
                method: &method,
                uri: &uri,
            };

            phenomenon_http_signatures::verify(request, public_key.to_pkcs1_der()?.as_bytes())
                .map_err(Error::from)
        })
        .await??;

        if !is_valid {
            return Err(StatusCode::UNAUTHORIZED.into_response());
        }

        Ok(Self(activity))
    }
}

#[debug_handler]
pub async fn post(SignedActivity(activity): SignedActivity) {
    // TODO: Insert activity into database

    match activity.rest.r#type.as_str() {
        "Follow" => (),
        "Undo" => (),
        "Delete" => (),
        "Announce" => (),
        "Create" => (),
        "Like" => (),
        "Update" => (),
        _ => (),
    }
}
