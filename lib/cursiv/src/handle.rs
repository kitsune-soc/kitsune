use crate::{CsrfData, HashRef, Message, MessageRef, RANDOM_DATA_LEN};
use hex_simd::{AsOut, AsciiCase};
use rand::{Rng, distributions::Alphanumeric};
use std::{fmt::Display, sync::Mutex};
use triomphe::Arc;
use zeroize::{Zeroize, ZeroizeOnDrop};

pub struct Shared {
    pub(crate) read_data: Option<CsrfData>,
    pub(crate) set_data: Option<CsrfData>,
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct CsrfHandle {
    #[zeroize(skip)]
    pub(crate) inner: Arc<Mutex<Shared>>,
    pub(crate) key: [u8; blake3::KEY_LEN],
}

fn raw_verify(key: &[u8; blake3::KEY_LEN], hash: &HashRef, message: &MessageRef) -> bool {
    let (hash, message) = (hash.as_ref(), message.as_ref());
    if hash.len() / 2 != blake3::OUT_LEN {
        return false;
    }

    let mut decoded_hash = [0_u8; blake3::OUT_LEN];
    if hex_simd::decode(hash.as_bytes(), decoded_hash.as_mut().as_out()).is_err() {
        return false;
    }

    let expected_hash = blake3::keyed_hash(key, message.as_bytes());

    // The `PartialEq` implementation on `Hash` is constant-time
    expected_hash == decoded_hash
}

impl CsrfHandle {
    /// Keep the current signature and message inside the cookie
    #[inline]
    pub fn keep_cookie(&self) {
        let inner = &mut *self.inner.lock().unwrap();
        inner.set_data.clone_from(&inner.read_data);
    }

    /// Create a signature and store it inside a cookie
    ///
    /// **Important**: The data passed into this function should reference an *authenticated session*.
    /// The use of the user ID (or something similarly static) is *discouraged*, use the session ID.
    #[inline]
    pub fn sign<SID>(&self, session_id: SID) -> Message
    where
        SID: AsRef<[u8]> + Display,
    {
        let random = rand::thread_rng()
            .sample_iter(Alphanumeric)
            .map(char::from)
            .take(RANDOM_DATA_LEN)
            .collect::<String>();

        let message = format!("{session_id}!{random}");
        let hash = blake3::keyed_hash(&self.key, message.as_bytes());
        let hash = hex_simd::encode_to_string(hash.as_bytes(), AsciiCase::Lower);

        let message: Message = message.into();
        self.inner.lock().unwrap().set_data = Some(CsrfData {
            hash: hash.into(),
            message: message.clone(),
        });

        message
    }

    /// Verify the CSRF request
    ///
    /// Simply pass in the message that was submitted by the client.
    /// Internally, we will verify the keyed hash read from the CSRF cookie to the value stored in the cookie,
    /// and to the value passed to the function
    #[inline]
    #[must_use]
    pub fn verify(&self, message: &MessageRef) -> bool {
        let guard = self.inner.lock().unwrap();
        let Some(ref read_data) = guard.read_data else {
            return false;
        };

        raw_verify(&self.key, &read_data.hash, &read_data.message)
            && raw_verify(&self.key, &read_data.hash, message)
    }
}

#[cfg(feature = "axum")]
mod axum_impl {
    use super::CsrfHandle;
    use axum_core::extract::FromRequestParts;
    use http::request::Parts;
    use std::convert::Infallible;

    impl<S> FromRequestParts<S> for CsrfHandle
    where
        S: Sync,
    {
        type Rejection = Infallible;

        async fn from_request_parts(
            parts: &mut Parts,
            _state: &S,
        ) -> Result<Self, Self::Rejection> {
            let handle = parts
                .extensions
                .get::<Self>()
                .expect("Service not wrapped by CSRF middleware")
                .clone();

            Ok(handle)
        }
    }
}
