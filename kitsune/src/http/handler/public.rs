use axum::{extract::Path, routing, Router, TypedHeader};
use axum_extra::either::Either;
use headers::ContentType;
use http::StatusCode;
use include_dir::include_dir;
use mime::Mime;
use once_cell::sync::Lazy;
use std::{collections::HashMap, path::Path as FsPath, sync::RwLock};

static PUBLIC_DIR: include_dir::Dir<'_> = include_dir!("public");
static PUBLIC_DIR_MIME_TYPE: Lazy<RwLock<HashMap<&'static FsPath, Mime>>> =
    Lazy::new(RwLock::default);

#[allow(clippy::unused_async)]
async fn get(
    Path(path): Path<String>,
) -> Either<(TypedHeader<ContentType>, &'static [u8]), StatusCode> {
    let Some(file) = PUBLIC_DIR.get_file(path) else {
        return Either::E2(StatusCode::NOT_FOUND);
    };

    let mime_type = PUBLIC_DIR_MIME_TYPE
        .read()
        .unwrap()
        .get(file.path())
        .map(Mime::clone);

    let mime_type = if let Some(mime_type) = mime_type {
        mime_type
    } else {
        let mime_type = mime_guess::from_path(file.path()).first_or_octet_stream();
        PUBLIC_DIR_MIME_TYPE
            .write()
            .unwrap()
            .insert(file.path(), mime_type.clone());

        mime_type
    };

    Either::E1((TypedHeader(ContentType::from(mime_type)), file.contents()))
}

pub fn routes<T>() -> Router<T>
where
    T: Clone + Send + Sync + 'static,
{
    Router::new().route("/*path", routing::get(get))
}
