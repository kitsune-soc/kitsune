use axum::{
    response::{IntoResponseParts, ResponseParts},
    Json,
};
use http::{Error as HttpError, HeaderValue};
use std::{borrow::Cow, fmt::Display};

use crate::error::Error;

pub type PaginatedJsonResponse<T> = (
    Option<LinkHeader<Vec<(&'static str, String)>>>,
    Json<Vec<T>>,
);

pub struct LinkHeader<T>(pub T);

impl<T, K, V> IntoResponseParts for LinkHeader<T>
where
    T: IntoIterator<Item = (K, V)>,
    K: Display,
    V: Display,
{
    type Error = Error;

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        let value = self
            .0
            .into_iter()
            .map(|(key, value)| Cow::Owned(format!("<{value}>; rel=\"{key}\"")))
            .intersperse(Cow::Borrowed(", "))
            .collect::<String>();

        res.headers_mut().insert(
            "Link",
            HeaderValue::from_str(&value).map_err(HttpError::from)?,
        );

        Ok(res)
    }
}

impl LinkHeader<Vec<(&'static str, String)>> {
    pub fn new<I, D: Display, F: Fn(&I) -> D>(
        collection: &[I],
        limit: usize,
        base_url: &str,
        uri_path: &str,
        get_key: F,
    ) -> Option<LinkHeader<Vec<(&'static str, String)>>> {
        if collection.is_empty() {
            None
        } else {
            let next = (
                "next",
                format!(
                    "{}{}?limit={}&max_id={}",
                    base_url,
                    uri_path,
                    limit,
                    get_key(collection.last().unwrap())
                ),
            );
            let prev = (
                "prev",
                format!(
                    "{}{}?limit={}&since_id={}",
                    base_url,
                    uri_path,
                    limit,
                    get_key(collection.first().unwrap())
                ),
            );
            if collection.len() >= limit && limit > 0 {
                Some(LinkHeader(vec![next, prev]))
            } else {
                Some(LinkHeader(vec![prev]))
            }
        }
    }
}
