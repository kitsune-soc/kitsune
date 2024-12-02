use bytes::Bytes;
use headers::HeaderMapExt;

pub use self::error::{Error, Result};
pub use self::params::ParamStorage;

mod error;
mod params;

pub struct Authorizer<'a> {
    request: &'a http::Request<Bytes>,
    query: ParamStorage<&'a str, &'a str>,
    body: ParamStorage<&'a str, &'a str>,
}

impl<'a> Authorizer<'a> {
    pub fn extract(req: &'a http::Request<Bytes>) -> Result<Self> {
        let body = req.body();

        // TECHNICALLY the body should only be URL-encoded.
        // Practically implementations like Mastodon and Pleroma allow it.
        // Therefore I really don't care what some weird nerds say, this is going in.
        let body = if req.headers().typed_get::<headers::ContentType>()
            == Some(headers::ContentType::json())
        {
            sonic_rs::from_slice(body).map_err(Error::body)
        } else {
            serde_urlencoded::from_bytes(body).map_err(Error::body)
        }?;

        let query = if let Some(raw_query) = req.uri().query() {
            serde_urlencoded::from_str(raw_query).map_err(Error::query)?
        } else {
            ParamStorage::new()
        };

        Ok(Self {
            body,
            query,
            request: req,
        })
    }

    pub async fn accept(self) -> http::Response<()> {
        todo!();
    }

    pub async fn deny(self) -> http::Response<()> {
        todo!();
    }
}
