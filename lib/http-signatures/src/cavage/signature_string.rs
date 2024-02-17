use super::SignatureHeader;
use crate::{REQUIRED_GET_HEADERS, REQUIRED_POST_HEADERS};
use http::Method;
use miette::Diagnostic;
use std::fmt::Write;
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
pub enum Error {
    #[error(transparent)]
    InvalidHeaderValue(#[from] http::header::ToStrError),

    #[error("Invalid HTTP method")]
    InvalidMethod,

    #[error("Missing required header names")]
    MissingHeaderNames,

    #[error("Missing header value")]
    MissingHeaderValue,
}

#[inline]
fn is_subset<I>(left: &[I], right: &[I]) -> bool
where
    I: PartialEq,
{
    if left.len() <= right.len() {
        left.iter().all(|item| right.contains(item))
    } else {
        false
    }
}

#[inline]
pub fn construct<'a, B, I>(
    request: &http::Request<B>,
    signature_header: &SignatureHeader<'_, I>,
) -> Result<String, Error>
where
    I: Iterator<Item = &'a str> + Clone,
{
    let header_names = signature_header.headers.clone().collect::<Vec<&str>>();
    let fulfills_min_requirements = match *request.method() {
        Method::GET => is_subset(REQUIRED_GET_HEADERS, &header_names),
        Method::POST => is_subset(REQUIRED_POST_HEADERS, &header_names),
        _ => return Err(Error::InvalidMethod),
    };

    if !fulfills_min_requirements {
        return Err(Error::MissingHeaderNames);
    }

    let mut signature_string = String::new();
    for name in header_names {
        match name {
            name @ "(request-target)" => {
                let method = request.method().as_str().to_lowercase();
                let path_and_query = request.uri().path_and_query().map_or_else(
                    || request.uri().path(),
                    |path_and_query| path_and_query.as_str(),
                );

                let _ = writeln!(signature_string, "{name}: {method} {path_and_query}");
            }
            name @ "(created)" => {
                let created = signature_header.created.ok_or(Error::MissingHeaderValue)?;
                let _ = writeln!(signature_string, "{name}: {created}");
            }
            name @ "(expires)" => {
                let expires = signature_header.expires.ok_or(Error::MissingHeaderValue)?;
                let _ = writeln!(signature_string, "{name}: {expires}");
            }
            header => {
                let value = request
                    .headers()
                    .get(header)
                    .ok_or(Error::MissingHeaderValue)?
                    .to_str()?;

                let _ = writeln!(signature_string, "{}: {}", header.to_lowercase(), value);
            }
        }
    }

    signature_string.shrink_to_fit();

    Ok(signature_string)
}

#[cfg(test)]
mod test {
    use super::is_subset;
    use proptest::{prop_assert_eq, proptest};
    use std::collections::HashSet;

    proptest! {
        #[test]
        fn subset_behaves_equal(left: HashSet<String>, right: HashSet<String>) {
            let vec_left = left.iter().collect::<Vec<_>>();
            let vec_right = right.iter().collect::<Vec<_>>();

            let slice_subset = is_subset(&vec_left, &vec_right);
            let set_subset = left.is_subset(&right);

            prop_assert_eq!(slice_subset, set_subset);
        }
    }
}
