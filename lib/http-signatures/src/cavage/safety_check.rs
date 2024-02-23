use super::SignatureHeader;
use http::{header::DATE, Method, Request};
use miette::Diagnostic;
use std::{
    cmp::min,
    time::{Duration, SystemTime, SystemTimeError},
};
use thiserror::Error;

/// 1 minute
const CLOCK_SKEW_ADJUSTMENT: Duration = Duration::from_secs(60);

/// 15 minutes
const MAX_ACCEPTED_SIGNATURE_AGE: Duration = Duration::from_secs(15 * 60);

const REQUIRED_GET_HEADERS: &[&str] = &["host"];
const REQUIRED_POST_HEADERS: &[&str] = &["host", "content-type", "digest"];

/// Safety check error
#[derive(Debug, Diagnostic, Error)]
pub enum SafetyCheckError {
    /// `Date` header has an invalid format
    #[error(transparent)]
    InvalidDateHeader(#[from] httpdate::Error),

    /// Header value is invalid (non UTF-8 value)
    #[error(transparent)]
    InvalidHeaderValue(#[from] http::header::ToStrError),

    /// `SystemTime` operation failed
    #[error(transparent)]
    InvalidSystemTime(#[from] SystemTimeError),

    /// Missing one of the required headers in the signature
    #[error("Missing required headers")]
    MissingRequiredHeaders,

    /// Signature is too old and thus invalid
    #[error("Signature too old")]
    SignatureTooOld,

    /// The HTTP method of the request is unsupported
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

/// Perform a basic safety check
///
/// This safety check includes whether:
///
/// - the age of the signature is outside the acceptable range
/// - the minimum of headers to ensure authenticity are included in the signature
#[inline]
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

    // Move all of the timestamps a minute into the future to compensate for our local clock maybe lagging behind a bit
    let now = SystemTime::now() + CLOCK_SKEW_ADJUSTMENT;
    let signature_valid_duration = if let Some(expires) = signature_header.expires {
        let expiration_timestamp =
            SystemTime::UNIX_EPOCH + Duration::from_secs(expires) + CLOCK_SKEW_ADJUSTMENT;

        min(
            expiration_timestamp.duration_since(SystemTime::now())?,
            MAX_ACCEPTED_SIGNATURE_AGE,
        )
    } else {
        MAX_ACCEPTED_SIGNATURE_AGE
    };

    if let Some(created) = signature_header.created {
        let created_time = SystemTime::UNIX_EPOCH + Duration::from_secs(created);
        if now.duration_since(created_time)? > signature_valid_duration {
            return Err(SafetyCheckError::SignatureTooOld);
        }
    }

    if let Some(date_header) = req.headers().get(DATE) {
        let date_header_time = httpdate::parse_http_date(date_header.to_str()?)?;
        if now.duration_since(date_header_time)? > signature_valid_duration {
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
