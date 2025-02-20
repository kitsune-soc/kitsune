use axum::extract::Path;
use axum_extra::{TypedHeader, either::Either};
use headers::ContentType;
use http::StatusCode;
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "assets"]
struct AssetsDir;

#[allow(clippy::unused_async)]
pub async fn get(
    Path(path): Path<String>,
) -> Either<(TypedHeader<ContentType>, Cow<'static, [u8]>), StatusCode> {
    let Some(file) = AssetsDir::get(&path) else {
        return Either::E2(StatusCode::NOT_FOUND);
    };
    let mime_type = mime_guess::from_path(&path).first_or_octet_stream();

    Either::E1((TypedHeader(ContentType::from(mime_type)), file.data))
}
