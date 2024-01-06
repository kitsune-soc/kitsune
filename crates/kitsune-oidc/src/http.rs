use http::Request;
use http_body_util::Full;
use http_compat::Compat;
use kitsune_http_client::Client as HttpClient;
use once_cell::sync::Lazy;
use openidconnect::{HttpRequest, HttpResponse};

static HTTP_CLIENT: Lazy<HttpClient> = Lazy::new(HttpClient::default);

pub async fn async_client(req: HttpRequest) -> Result<HttpResponse, kitsune_http_client::Error> {
    let mut request = Request::builder()
        .method(req.method.compat())
        .uri(req.url.as_str());
    *request.headers_mut().unwrap() = req.headers.compat();
    let request = request.body(Full::from(req.body)).unwrap();
    let response = HTTP_CLIENT.execute(request).await?;

    Ok(HttpResponse {
        status_code: response.status().compat(),
        headers: response.headers().clone().compat(),
        body: response.bytes().await?.to_vec(),
    })
}
