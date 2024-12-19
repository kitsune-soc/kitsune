use crate::{error::Error, params::ParamStorage};
use bytes::Bytes;
use memchr::memchr;

static URL_ENCODED_CONTENT_TYPE: http::HeaderValue =
    http::HeaderValue::from_static("application/x-www-form-urlencoded");

#[inline]
pub fn body<'a, T>(req: &'a http::Request<Bytes>) -> Result<T, Error>
where
    T: serde::Deserialize<'a>,
{
    // Not part of the RFC, but a bunch of implementations allow this.
    // And because they allow this, clients make use of this.
    //
    // Done to increase compatibility.
    let content_type = req.headers().get(http::header::CONTENT_TYPE);
    if content_type == Some(&URL_ENCODED_CONTENT_TYPE) {
        serde_urlencoded::from_bytes(req.body()).map_err(Error::body)
    } else {
        sonic_rs::from_slice(req.body()).map_err(Error::body)
    }
}

pub enum ClientCredentials<'a> {
    Basic(BasicAuth),
    Body {
        client_id: &'a str,
        client_secret: &'a str,
    },
}

impl<'a> ClientCredentials<'a> {
    #[inline]
    #[instrument(skip_all)]
    pub fn extract(headers: &http::HeaderMap, body: &ParamStorage<&str, &'a str>) -> Option<Self> {
        if let Some(auth) = BasicAuth::extract(headers) {
            Some(Self::Basic(auth))
        } else {
            debug!("attempting to read client credentials from body (naughty :3)");

            // As a fallback, try to read from the body.
            // Not recommended but some clients do this. Done to increase compatibility.

            let client_id = body.get("client_id")?;
            let client_secret = body.get("client_secret")?;

            Some(Self::Body {
                client_id,
                client_secret,
            })
        }
    }

    #[inline]
    #[must_use]
    pub fn client_id(&self) -> &str {
        match self {
            Self::Basic(auth) => auth.username(),
            Self::Body { client_id, .. } => client_id,
        }
    }

    #[inline]
    #[must_use]
    pub fn client_secret(&self) -> &str {
        match self {
            Self::Basic(auth) => auth.password(),
            Self::Body { client_secret, .. } => client_secret,
        }
    }
}

pub struct BasicAuth {
    buffer: String,
    delimiter_pos: usize,
}

impl BasicAuth {
    #[inline]
    pub fn extract(headers: &http::HeaderMap) -> Option<Self> {
        let auth = headers.get(http::header::AUTHORIZATION)?;
        let auth_bytes = auth.as_bytes();

        let space_location = memchr(b' ', auth_bytes)?;
        let method = &auth_bytes[..space_location];
        let value = &auth_bytes[(space_location + 1)..];

        if method != b"Basic" {
            return None;
        }

        let buffer = base64_simd::STANDARD
            .decode_to_vec(value)
            .inspect_err(|error| debug!(?error, "failed to decode basic auth"))
            .ok()?;

        // SAFETY: Since `simdutf8` validates that the buffer contents are valid UTF-8 and we exit the function on error,
        // we can simply call `String::from_utf8_unchecked`.
        #[allow(unsafe_code)]
        let buffer = unsafe {
            simdutf8::basic::from_utf8(&buffer)
                .inspect_err(|error| debug!(?error, "failed to decode utf8"))
                .ok()?;

            String::from_utf8_unchecked(buffer)
        };

        let delimiter_pos = memchr(b':', buffer.as_bytes())?;

        Some(Self {
            buffer,
            delimiter_pos,
        })
    }

    #[inline]
    #[must_use]
    pub fn username(&self) -> &str {
        // SAFETY: The delimiter was previously found via `str::find`, so the index is guaranteed to be within boundaries
        #[allow(unsafe_code)]
        unsafe {
            self.buffer.get_unchecked(..self.delimiter_pos)
        }
    }

    #[inline]
    #[must_use]
    pub fn password(&self) -> &str {
        // SAFETY: The delimiter was previously found via `str::find`, so the index is guaranteed to be within boundaries
        #[allow(unsafe_code)]
        unsafe {
            self.buffer.get_unchecked((self.delimiter_pos + 1)..)
        }
    }
}
