use kitsune_http_client::Client as HttpClient;
use once_cell::sync::Lazy;
use openidconnect::{HttpRequest, HttpResponse};

static HTTP_CLIENT: Lazy<HttpClient> = Lazy::new(HttpClient::default);

pub async fn async_client(req: HttpRequest) -> Result<HttpResponse, kitsune_http_client::Error> {
    let response = HTTP_CLIENT.execute(req.map(Into::into)).await?;

    let mut builder = http::Response::builder()
        .status(response.status())
        .version(response.version());
    *builder.headers_mut().unwrap() = response.headers().clone();

    Ok(builder.body(response.bytes().await?.to_vec()).unwrap())
}
