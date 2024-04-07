use axum::extract::multipart;
use bytes::Bytes;
use color_eyre::eyre::Context;
use futures_util::{Stream, TryStreamExt};
use kitsune_error::Result;
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
) -> Result<impl Stream<Item = Result<Bytes>> + Send + 'static> {
    let tempfile = tempfile().unwrap();
    let mut tempfile = File::from_std(tempfile);

    while let Some(chunk) = field.chunk().await? {
        tempfile
            .write_all(&chunk)
            .await
            .wrap_err("failed to write chunk to tempfile")?;
    }

    tempfile.seek(SeekFrom::Start(0)).await?;

    Ok(ReaderStream::new(tempfile).map_err(Into::into))
}
