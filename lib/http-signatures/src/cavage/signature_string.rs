//!
//! Utilities for handling signature strings
//!

use super::SignatureHeader;
use miette::Diagnostic;
use std::fmt::Write;
use thiserror::Error;

/// Signature string error
#[derive(Debug, Diagnostic, Error)]
pub enum Error {
    /// Header had an invalid value (non-UTF8 value)
    #[error(transparent)]
    InvalidHeaderValue(#[from] http::header::ToStrError),

    /// Header is missing from the request
    #[error("Missing header value")]
    MissingHeaderValue,
}

/// Construct a new signature string from a parsed signature header and an HTTP request
#[inline]
pub fn construct<'a, B, I, S>(
    request: &http::Request<B>,
    signature_header: &SignatureHeader<'_, I, S>,
) -> Result<String, Error>
where
    I: Iterator<Item = &'a str> + Clone,
{
    let mut signature_string = String::new();
    for name in signature_header.headers.clone() {
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

    // Remove the last new-line
    signature_string.pop();

    Ok(signature_string)
}

#[cfg(test)]
mod test {
    use http::{Method, Request, Uri};

    const BASIC_SIGNATURE_STRING: &str = "(request-target): get /foo?param=value&pet=dog\nhost: example.com\ndate: Sun, 05 Jan 2014 21:31:40 GMT";
    const ALL_HEADERS_SIGNATURE_STRING: &str = "(request-target): post /foo?param=value&pet=dog\nhost: example.com\ndate: Sun, 05 Jan 2014 21:31:40 GMT\ncontent-type: application/json\ndigest: SHA-256=X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=\ncontent-length: 18";

    fn request(method: Method) -> Request<()> {
        Request::builder()
            .method(method)
            .uri(Uri::from_static("/foo?param=value&pet=dog"))
            .header("Host", "example.com")
            .header("Date", "Sun, 05 Jan 2014 21:31:40 GMT")
            .header("Content-Type", "application/json")
            .header(
                "Digest",
                "SHA-256=X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=",
            )
            .header("Content-Length", "18")
            .body(())
            .unwrap()
    }

    #[test]
    fn basic_signature_string() {
        let request = request(Method::GET);
        let signature_header = crate::cavage::parse(r#"keyId="Test",algorithm="rsa-sha256",headers="(request-target) host date",signature="qdx""#).unwrap();
        let signature_string = super::construct(&request, &signature_header).unwrap();

        assert_eq!(signature_string, BASIC_SIGNATURE_STRING);
    }

    #[test]
    #[ignore = "Test vector is broken"] // Lol. Lmao even.
    fn all_headers_signature_string() {
        let request = request(Method::POST);
        let signature_header = crate::cavage::parse(r#"keyId="Test",algorithm="rsa-sha256",created=1402170695, expires=1402170699,headers="(request-target) (created) (expires) host date content-type digest content-length",signature="vSdr""#).unwrap();
        let signature_string = super::construct(&request, &signature_header).unwrap();

        assert_eq!(signature_string, ALL_HEADERS_SIGNATURE_STRING);
    }
}
