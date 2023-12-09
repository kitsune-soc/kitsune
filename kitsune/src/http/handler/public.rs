use axum::{extract::Path, routing, Router, TypedHeader};
use axum_extra::either::Either;
use headers::ContentType;
use http::StatusCode;
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "assets-dist"]
#[exclude = "*.scss"]
struct AssetsDir;

#[allow(clippy::unused_async)]
async fn get(
    Path(path): Path<String>,
) -> Either<(TypedHeader<ContentType>, Cow<'static, [u8]>), StatusCode> {
    let Some(file) = AssetsDir::get(&path) else {
        return Either::E2(StatusCode::NOT_FOUND);
    };
    let mime_type = mime_guess::from_path(&path).first_or_octet_stream();

    Either::E1((TypedHeader(ContentType::from(mime_type)), file.data))
}

pub fn routes<T>() -> Router<T>
where
    T: Clone + Send + Sync + 'static,
{
    Router::new().route("/*path", routing::get(get))
}
