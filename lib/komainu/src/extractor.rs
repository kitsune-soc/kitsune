use crate::{
    error::{Error, Result},
    params::ParamStorage,
    OptionExt,
};
use bytes::Bytes;

static URL_ENCODED_CONTENT_TYPE: http::HeaderValue =
    http::HeaderValue::from_static("application/x-www-form-urlencoded");

#[inline]
pub fn body<'a, T>(req: &'a http::Request<Bytes>) -> Result<T>
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
    pub fn client_id(&self) -> &str {
        match self {
            Self::Basic(auth) => auth.username(),
            Self::Body { client_id, .. } => client_id,
        }
    }

    #[inline]
    pub fn client_secret(&self) -> &str {
        match self {
            Self::Basic(auth) => auth.password(),
            Self::Body { client_secret, .. } => client_secret,
        }
    }
}

pub struct BasicAuth {
    buffer: Vec<u8>,
    username: &'static str,
    password: &'static str,
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

        let buffer_str = simdutf8::basic::from_utf8(&value)
            .inspect_err(|error| debug!(?error, "failed to decode utf8"))
            .ok()?;

        let (username, password) = buffer_str.split_once(':')?;

        // SAFETY: self-referential struct. can't access invariant lifetimes from the outside.
        #[allow(unsafe_code)]
        unsafe {
            Some(Self {
                buffer,
                username: std::mem::transmute(username),
                password: std::mem::transmute(password),
            })
        }
    }

    #[inline]
    pub fn username(&self) -> &str {
        &self.username
    }

    #[inline]
    pub fn password(&self) -> &str {
        &self.password
    }
}

#[cfg(test)]
mod test {
    use super::BasicAuth;

    #[test]
    fn parse_basic_auth_rfc() {
        let mut map = http::HeaderMap::new();
        map.insert(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_static("Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ=="),
        );

        let auth = BasicAuth::extract(&map).unwrap();
        assert_eq!(auth.username(), "Aladdin");
        assert_eq!(auth.password(), "open sesame");
    }
}
