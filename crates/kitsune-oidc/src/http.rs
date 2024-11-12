use crate::error::ErrorWrapper;
use http_body_util::BodyExt;
use kitsune_http_client::Client as HttpClient;
use openidconnect::{HttpRequest, HttpResponse};
use std::sync::LazyLock;

static HTTP_CLIENT: LazyLock<HttpClient> = LazyLock::new(HttpClient::default);

#[inline]
pub async fn async_client(req: HttpRequest) -> Result<HttpResponse, ErrorWrapper> {
    let response = HTTP_CLIENT
        .execute(req.map(Into::into))
        .await
        .map_err(|err| ErrorWrapper(err.into()))?;

    let (parts, body) = response.into_inner().into_parts();
    let body = body.collect().await.map_err(ErrorWrapper)?.to_bytes();

    Ok(HttpResponse::from_parts(parts, body.to_vec()))
}
