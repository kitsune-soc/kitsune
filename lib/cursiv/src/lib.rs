#![doc = include_str!("../README.md")]

pub use self::{
    future::ResponseFuture, handle::CsrfHandle, layer::CsrfLayer, newtypes::*, service::CsrfService,
};

mod future;
mod handle;
mod layer;
mod service;

const CSRF_COOKIE_NAME: &str = "CURSIV-CSRF_TOKEN";
const RANDOM_DATA_LEN: usize = 32;

mod newtypes {
    #[aliri_braid::braid]
    pub struct Hash;

    #[aliri_braid::braid]
    pub struct Message;
}

#[derive(Clone)]
struct CsrfData {
    hash: Hash,
    message: Message,
}
