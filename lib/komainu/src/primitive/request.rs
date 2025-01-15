use crate::{
    error::{BoxError, Error},
    params::ParamStorage,
};
use http_body_util::BodyExt;
use std::borrow::Cow;

pub struct Request<'a> {
    pub headers: http::HeaderMap,
    pub query: ParamStorage<Cow<'a, str>, Cow<'a, str>>,
    pub body: ParamStorage<Cow<'a, str>, Cow<'a, str>>,
}

impl Request<'_> {
    #[inline]
    #[cfg_attr(not(coverage), instrument(skip_all))]
    pub async fn read_from<B>(req: http::Request<B>) -> Result<Self, Error>
    where
        B: http_body::Body,
        B::Error: Into<BoxError>,
    {
        let raw_query = req.uri().query().unwrap_or("");
        let query = serde_urlencoded::from_str(raw_query).map_err(Error::query)?;

        let (parts, body) = req.into_parts();
        let collected = body.collect().await.map_err(Error::body)?.to_bytes();
        let req = http::Request::from_parts(parts, collected.clone());

        let body = if collected.is_empty() {
            ParamStorage::default()
        } else {
            crate::extract::body(&req).map_err(Error::body)?
        };

        Ok(Self {
            headers: req.headers().clone(),
            query,
            body,
        })
    }
}
