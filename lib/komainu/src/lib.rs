pub use self::error::{Error, Result};

mod error;
mod params;

pub struct Authorizer<C> {
    _owo: C,
}

impl<C> Authorizer<C> {
    pub async fn extract<B>(req: http::Request<B>) -> Result<Self> {
        todo!();
    }

    pub async fn accept(self) -> http::Response<()> {
        todo!();
    }

    pub async fn deny(self) -> http::Response<()> {
        todo!();
    }
}
