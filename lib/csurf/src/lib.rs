use hex_simd::{AsciiCase, Out};
use http::{Request, Response};
use rand::RngCore;
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
    task::{self, Poll},
};
use tower::{Layer, Service};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[aliri_braid::braid]
pub struct Hash;

#[aliri_braid::braid]
pub struct Message;

struct Shared {
    read_data: Option<(Hash, Message)>,
    set_data: Option<(Hash, Message)>,
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct CsrfHandle {
    #[zeroize(skip)]
    inner: Arc<Mutex<Shared>>,
    key: [u8; blake3::KEY_LEN],
}

fn raw_verify(key: &[u8; blake3::KEY_LEN], hash: &HashRef, message: &MessageRef) -> bool {
    let (hash, message) = (hash.as_ref(), message.as_ref());
    if hash.len() / 2 != blake3::OUT_LEN {
        return false;
    }

    let mut decoded_hash = [0_u8; blake3::OUT_LEN];
    if hex_simd::decode(hash.as_bytes(), Out::from_slice(&mut decoded_hash)).is_err() {
        return false;
    }

    let expected_hash = blake3::keyed_hash(key, message.as_bytes());

    // The `PartialEq` implementation on `Hash` is constant-time
    expected_hash == decoded_hash
}

impl CsrfHandle {
    pub fn sign<SID>(&self, session_id: SID) -> Message
    where
        SID: AsRef<[u8]> + Display,
    {
        let mut buf = [0; 16];
        rand::thread_rng().fill_bytes(&mut buf);
        let random = hex_simd::encode_to_string(buf, AsciiCase::Lower);

        let message = format!("{session_id}!{random}");
        let hash = blake3::keyed_hash(&self.key, message.as_bytes());
        let hash = hex_simd::encode_to_string(hash.as_bytes(), AsciiCase::Lower);

        let message: Message = message.into();
        self.inner.lock().unwrap().set_data = Some((hash.into(), message.clone()));

        message
    }

    #[must_use]
    pub fn verify(&self, message: &MessageRef) -> bool {
        let guard = self.inner.lock().unwrap();
        let Some(ref read_data) = guard.read_data else {
            return false;
        };

        if !raw_verify(&self.key, &read_data.0, &read_data.1) {
            return false;
        }

        raw_verify(&self.key, &read_data.0, message)
    }
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct CsrfLayer {
    key: [u8; blake3::KEY_LEN],
}

impl<S> Layer<S> for CsrfLayer {
    type Service = CsrfService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CsrfService::new(inner, self.key)
    }
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct CsrfService<S> {
    #[zeroize(skip)]
    inner: S,
    key: [u8; blake3::KEY_LEN],
}

impl<S> CsrfService<S> {
    pub fn new(inner: S, key: [u8; blake3::KEY_LEN]) -> Self {
        Self { inner, key }
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for CsrfService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Error = S::Error;
    type Future = S::Future;
    type Response = S::Response;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        self.inner.call(req)
    }
}
