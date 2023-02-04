use self::{auth::AuthMutation, posts::PostMutation, users::UserMutation};
use crate::{db::model::media_attachment, http::graphql::ContextExt};
use async_graphql::{Context, Error, MergedObject, Result, Upload};
use chrono::Utc;
use image::{EncodableLayout, GenericImageView};
use mime::Mime;
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use tokio::{
    fs::{self, File},
    io,
};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use uuid::Uuid;

mod auth;
mod post;
mod user;

const ALLOWED_FILETYPES: &[mime::Name<'_>] = &[mime::IMAGE, mime::VIDEO, mime::AUDIO];

async fn calculate_blurhash<P>(path: P) -> Result<String>
where
    P: AsRef<Path> + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        let image = image::open(path)?;
        let (width, height) = image.dimensions();
        let rgba_data = image.into_rgba8();

        Ok(blurhash_ng::encode(
            4,
            3,
            width,
            height,
            rgba_data.as_bytes(),
        ))
    })
    .await?
}

/// Saves the file into a user-configured subdirectory and returns a full URL to the file
// TODO: Refactor this
async fn handle_upload(
    ctx: &Context<'_>,
    file: Upload,
    description: Option<String>,
) -> Result<media_attachment::Model> {
    let state = ctx.state();
    let user_data = ctx.user_data()?;
    let value = file.value(ctx)?;
    let content_type = value
        .content_type
        .as_deref()
        .and_then(|content_type| Mime::from_str(content_type).ok())
        .or_else(|| mime_guess::from_path(&value.filename).first())
        .ok_or_else(|| Error::new("Failed to determine file type"))?;

    if !ALLOWED_FILETYPES.contains(&content_type.type_()) {
        return Err(Error::new("File type not allowed"));
    }

    // Create a directory with the name of a random UUID and place the file with its original filename into the directory.
    // Doing this will prevent virtually all cases of filename collissions.
    // The possibility of someone guessing the next UUID *and* knowing the name of the file is vanishingly small.
    let directory_name = PathBuf::from(Uuid::now_v7().to_string());

    let mut media_directory = state.config.upload_dir.clone();
    media_directory.push(&directory_name);
    fs::create_dir(&media_directory).await?;

    let mut relative_media_path = directory_name.clone();
    relative_media_path.push(&value.filename);

    let mut full_media_path = media_directory;
    full_media_path.push(&value.filename);

    let mut reader = value.into_async_read().compat();
    let mut writer = File::create(&full_media_path).await?;

    io::copy(&mut reader, &mut writer).await?;

    let url = format!(
        "https://{}/media/{}",
        state.config.domain,
        relative_media_path.display()
    );

    // TODO: Calculate blurhashes for image attachments
    let blurhash = if content_type.type_() == mime::IMAGE {
        calculate_blurhash(full_media_path).await.ok()
    } else {
        None
    };

    Ok(media_attachment::Model {
        id: Uuid::now_v7(),
        account_id: user_data.account.id,
        blurhash,
        content_type: content_type.to_string(),
        description,
        url,
        created_at: Utc::now(),
    }
    .into_active_model()
    .insert(&state.db_conn)
    .await?)
}

#[derive(Default, MergedObject)]
pub struct RootMutation(AuthMutation, PostMutation, UserMutation);
