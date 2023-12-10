use cookie::Cookie;
use hex_simd::{AsciiCase, Out};
use http::{Request, Response};
use pin_project_lite::pin_project;
use rand::RngCore;
use std::{
    fmt::Display,
    future::Future,
    pin::Pin,
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

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
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

pin_project! {
    pub struct ResponseFuture<F> {
        #[pin]
        inner: F,
        handle: CsrfHandle,
    }
}

impl<F, E, ResBody> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        this.inner.poll(cx).map_ok(|_resp| {
            todo!();
        })
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
    type Future = ResponseFuture<S::Future>;
    type Response = S::Response;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let handle = CsrfHandle {
            inner: Arc::new(Mutex::new(Shared {
                read_data: None,
                set_data: None,
            })),
            key: self.key,
        };

        req.extensions_mut().insert(handle.clone());

        ResponseFuture {
            inner: self.inner.call(req),
            handle,
        }
    }
}
