use std::{convert::Infallible, fmt::Display};

use axum::{
    response::{IntoResponseParts, ResponseParts},
    Json,
};
use http::HeaderValue;

pub type PaginatedJsonResponse<T> = (Option<Link<Vec<(&'static str, String)>>>, Json<Vec<T>>);

pub struct Link<T>(pub T);

impl<T, K, V> IntoResponseParts for Link<T>
where
    T: IntoIterator<Item = (K, V)>,
    K: Display,
    V: Display,
{
    type Error = Infallible;

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        let value = self
            .0
            .into_iter()
            .map(|(key, value)| format!("<{value}>; rel=\"{key}\""))
            .collect::<Vec<String>>()
            .join(", ");

        // TODO: log an error when HeaderValue conversion fails
        if let Ok(header_value) = HeaderValue::from_str(&value) {
            res.headers_mut().insert("Link", header_value);
        }

        Ok(res)
    }
}

pub fn new_link_header<I, D: Display, F: Fn(&I) -> D>(
    collection: &Vec<I>,
    limit: usize,
    base_url: &str,
    get_key: F,
) -> Option<Link<Vec<(&'static str, String)>>> {
    if collection.is_empty() {
        None
    } else {
        let next = (
            "next",
            format!(
                "{}/api/v1/timelines/public?limit={}&max_id={}",
                base_url,
                limit,
                get_key(collection.last().unwrap())
            ),
        );
        let prev = (
            "prev",
            format!(
                "{}/api/v1/timelines/public?limit={}&since_id={}",
                base_url,
                limit,
                get_key(collection.first().unwrap())
            ),
        );
        if collection.len() >= limit && limit > 0 {
            Some(Link(vec![next, prev]))
        } else {
            Some(Link(vec![prev]))
        }
    }
}
