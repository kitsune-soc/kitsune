use self::{auth::AuthMutation, post::PostMutation, user::UserMutation};
use crate::{http::graphql::ContextExt, service::attachment};
use async_graphql::{Context, Error, MergedObject, Result, Upload};
use futures_util::TryStreamExt;
use kitsune_db::model::media_attachment::MediaAttachment;
use mime::Mime;
use std::str::FromStr;
use tokio_util::{compat::FuturesAsyncReadCompatExt, io::ReaderStream};

mod auth;
mod post;
mod user;

/// Saves the file into a user-configured subdirectory and returns a full URL to the file
async fn handle_upload(
    ctx: &Context<'_>,
    file: Upload,
    description: Option<String>,
) -> Result<MediaAttachment> {
    let user_data = ctx.user_data()?;
    let value = file.value(ctx)?;
    let content_type = value
        .content_type
        .as_deref()
        .and_then(|content_type| Mime::from_str(content_type).ok())
        .or_else(|| mime_guess::from_path(&value.filename).first())
        .ok_or_else(|| Error::new("Failed to determine file type"))?;

    let stream = ReaderStream::new(value.into_async_read().compat()).map_err(Into::into);
    let mut upload = attachment::Upload::builder()
        .account_id(user_data.account.id)
        .content_type(content_type.as_ref().into())
        .stream(stream);

    if let Some(description) = description {
        upload = upload.description(description);
    }

    ctx.state()
        .service
        .attachment
        .upload(upload.build().unwrap())
        .await
        .map_err(Into::into)
}

#[derive(Default, MergedObject)]
pub struct RootMutation(AuthMutation, PostMutation, UserMutation);
