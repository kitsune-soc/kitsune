use crate::error::Result;
use axum::extract::multipart;
use bytes::Bytes;
use futures_util::{Stream, TryStreamExt};
use kitsune_core::error::HttpError;
use kitsune_storage::BoxError;
use std::io::SeekFrom;
use tempfile::tempfile;
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
};
use tokio_util::io::ReaderStream;

#[allow(dead_code)] // Not used when the Mastodon API feature is deactivated
pub async fn buffer_multipart_to_tempfile(
    field: &mut multipart::Field<'_>,
) -> Result<impl Stream<Item = Result<Bytes, BoxError>> + Send + 'static> {
    let tempfile = tempfile().unwrap();
    let mut tempfile = File::from_std(tempfile);

    while let Some(chunk) = field.chunk().await? {
        if let Err(error) = tempfile.write_all(&chunk).await {
            error!(?error, "Failed to write chunk to tempfile");
            return Err(HttpError::InternalServerError.into());
        }
    }

    tempfile.seek(SeekFrom::Start(0)).await.unwrap();

    Ok(ReaderStream::new(tempfile).map_err(Into::into))
}
