use std::{convert::Infallible, fmt::Display};

use axum::{
    response::{IntoResponseParts, ResponseParts},
    Json,
};
use http::HeaderValue;

pub type PaginatedJsonResponse<T> = (Option<Link<Vec<(String, String)>>>, Json<Vec<T>>);

pub struct Link<T>(pub T);

impl<T, S> IntoResponseParts for Link<T>
where
    T: IntoIterator<Item = (S, S)>,
    S: Display,
{
    type Error = Infallible;

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        let value = self
            .0
            .into_iter()
            .map(|(key, value)| format!("<{value}>; rel=\"{key}\""))
            .collect::<Vec<String>>()
            .join(", ");

        // as long as we pass valid links this should never panic
        res.headers_mut()
            .insert("Link", HeaderValue::from_str(&value).unwrap());

        Ok(res)
    }
}
