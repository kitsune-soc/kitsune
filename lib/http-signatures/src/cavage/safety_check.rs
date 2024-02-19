use http::{header::DATE, Method, Request};
use std::time::{Duration, SystemTime, SystemTimeError};
use thiserror::Error;

use super::SignatureHeader;

/// 15 minutes
const MAX_ACCEPTED_SIGNATURE_AGE: Duration = Duration::from_secs(15 * 60);

const REQUIRED_GET_HEADERS: &[&str] = &["host"];
const REQUIRED_POST_HEADERS: &[&str] = &["host", "content-type", "digest"];

#[derive(Debug, Error)]
pub enum SafetyCheckError {
    #[error(transparent)]
    InvalidDateHeader(#[from] httpdate::Error),

    #[error(transparent)]
    InvalidHeaderValue(#[from] http::header::ToStrError),

    #[error(transparent)]
    InvalidSystemTime(#[from] SystemTimeError),

    #[error("Missing required headers")]
    MissingRequiredHeaders,

    #[error("Signature too old")]
    SignatureTooOld,

    #[error("Unsupported HTTP method")]
    UnsupportedHttpMethod,
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

pub fn is_safe<'a, B, I, S>(
    req: &Request<B>,
    signature_header: &SignatureHeader<'_, I, S>,
) -> Result<(), SafetyCheckError>
where
    I: Iterator<Item = &'a str> + Clone,
{
    let collected_headers = signature_header.headers.clone().collect::<Vec<&str>>();
    let is_subset = match *req.method() {
        Method::GET => is_subset(REQUIRED_GET_HEADERS, &collected_headers),
        Method::POST => is_subset(REQUIRED_POST_HEADERS, &collected_headers),
        _ => return Err(SafetyCheckError::UnsupportedHttpMethod),
    };

    if !is_subset {
        return Err(SafetyCheckError::MissingRequiredHeaders);
    }

    // Check if the `headers` field either includes `date` or `(created)`
    if !collected_headers.contains(&"date") && !collected_headers.contains(&"(created)") {
        return Err(SafetyCheckError::MissingRequiredHeaders);
    }

    if let Some(created) = signature_header.created {
        let created_time = SystemTime::UNIX_EPOCH + Duration::from_secs(created);
        let now = SystemTime::now();

        // If there isn't a positive duration, simply set the duration to max to let the validation fail
        //
        // Tbh, if the clock scew of your server is that bad.. why do you even federate??
        if now.duration_since(created_time)? > MAX_ACCEPTED_SIGNATURE_AGE {
            return Err(SafetyCheckError::SignatureTooOld);
        }
    }

    if let Some(date_header) = req.headers().get(DATE) {
        let date_header_time = httpdate::parse_http_date(date_header.to_str()?)?;
        let now = SystemTime::now();

        if now.duration_since(date_header_time)? > MAX_ACCEPTED_SIGNATURE_AGE {
            return Err(SafetyCheckError::SignatureTooOld);
        }
    }

    Ok(())
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
