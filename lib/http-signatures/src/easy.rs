use http::HeaderName;
use tracing::{debug, instrument};

static SIGNATURE_HEADER: HeaderName = HeaderName::from_static("signature");

#[instrument(skip_all)]
pub async fn sign<B>(req: http::Request<B>) -> http::Request<B> {
    todo!();
}

#[instrument(skip_all)]
pub async fn verify<B>(req: &http::Request<B>) -> bool {
    let Some(header) = req.headers().get(&SIGNATURE_HEADER) else {
        debug!("Missing 'Signature' header");
        return false;
    };

    todo!();
}
