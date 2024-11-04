use crate::{
    ctx::Context,
    mrf_wit::v1::fep::mrf::http::{self, Error, Request, Response, ResponseBody},
};
use async_trait::async_trait;
use futures_util::TryStreamExt;
use http_body_util::{BodyDataStream, BodyExt};
use wasmtime::component::Resource;

pub type Body = BodyDataStream<kitsune_http_client::ResponseBody>;

#[async_trait]
impl http::Host for Context {
    async fn do_request(&mut self, request: Request) -> Result<Response, Resource<Error>> {
        let method = kitsune_http_client::http::Method::from_bytes(request.method.as_bytes())
            .map_err(|_| Resource::new_own(0))?;

        let request = kitsune_http_client::http::Request::builder()
            .uri(request.url)
            .method(method)
            .body(request.body.map_or_else(Default::default, Into::into))
            .unwrap();

        let response = self.http_ctx.client.execute(request).await.map_err(|e| {
            debug!(error = ?e, "http request failed");
            Resource::new_own(0)
        })?;

        let (parts, body) = response.into_inner().into_parts();
        let body = self.http_ctx.bodies.insert(body.into_data_stream());

        let headers = parts
            .headers
            .into_iter()
            .filter_map(|(maybe_key, value)| {
                let key = maybe_key?;
                Some((key.to_string(), value.to_str().ok()?.to_string()))
            })
            .collect();

        Ok(Response {
            status: parts.status.as_u16(),
            headers,
            body: Resource::new_own(body as u32),
        })
    }

    async fn do_request_signed(&mut self, _request: Request) -> Result<Response, Resource<Error>> {
        Err(Resource::new_own(0))
    }
}

#[async_trait]
impl http::HostResponseBody for Context {
    async fn next(
        &mut self,
        rep: Resource<ResponseBody>,
    ) -> Result<Option<Vec<u8>>, Resource<Error>> {
        let body = self.http_ctx.get_body(&rep);
        let chunk = body.try_next().await.map_err(|e| {
            debug!(error = ?e, "http response body failed");
            Resource::new_own(0)
        })?;

        Ok(chunk.map(|c| c.to_vec()))
    }

    async fn drop(&mut self, rep: Resource<ResponseBody>) -> wasmtime::Result<()> {
        self.http_ctx.bodies.remove(rep.rep() as usize);
        Ok(())
    }
}

#[async_trait]
impl http::HostError for Context {
    async fn drop(&mut self, _rep: Resource<Error>) -> wasmtime::Result<()> {
        Ok(())
    }
}
