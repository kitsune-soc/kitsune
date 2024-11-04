use crate::{
    ctx::Context,
    mrf_wit::v1::fep::mrf::http::{self, Error, Request, Response, ResponseBody},
};
use async_trait::async_trait;
use wasmtime::component::Resource;

#[async_trait]
impl http::Host for Context {
    async fn do_request(&mut self, request: Request) -> Result<Response, Resource<Error>> {
        todo!()
    }

    async fn do_request_signed(&mut self, request: Request) -> Result<Response, Resource<Error>> {
        todo!()
    }
}

#[async_trait]
impl http::HostResponseBody for Context {
    async fn next(
        &mut self,
        _rep: Resource<ResponseBody>,
    ) -> Result<Option<Vec<u8>>, Resource<Error>> {
        todo!()
    }

    async fn drop(&mut self, _rep: Resource<ResponseBody>) -> wasmtime::Result<()> {
        todo!()
    }
}

#[async_trait]
impl http::HostError for Context {
    async fn drop(&mut self, _rep: Resource<Error>) -> wasmtime::Result<()> {
        Ok(())
    }
}
