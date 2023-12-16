use crate::CsrfService;
use tower::Layer;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct CsrfLayer {
    key: [u8; blake3::KEY_LEN],
}

impl CsrfLayer {
    #[must_use]
    pub fn new(key: [u8; blake3::KEY_LEN]) -> Self {
        Self { key }
    }
}

impl<S> Layer<S> for CsrfLayer {
    type Service = CsrfService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CsrfService::new(inner, self.key)
    }
}
