use http::Request;
use hyper::Body;
use kitsune_http_client::Client as HttpClient;
use once_cell::sync::Lazy;
use openidconnect::{HttpRequest, HttpResponse};

static HTTP_CLIENT: Lazy<HttpClient> = Lazy::new(HttpClient::default);

#[allow(clippy::missing_panics_doc)]
pub async fn async_client(req: HttpRequest) -> Result<HttpResponse, kitsune_http_client::Error> {
    let mut request = Request::builder().method(req.method).uri(req.url.as_str());
    *request.headers_mut().unwrap() = req.headers;
    let request = request.body(Body::from(req.body)).unwrap();
    let response = HTTP_CLIENT.execute(request).await?;

    Ok(HttpResponse {
        status_code: response.status(),
        headers: response.headers().clone(),
        body: response.bytes().await?.to_vec(),
    })
}
