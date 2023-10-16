use crate::{
    consts::MAX_EMOJI_SHORTCODE_LENGTH,
    error::{Error, Result},
};

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::{Stream, TryStreamExt};
use garde::Validate;
use kitsune_db::{model::custom_emoji::CustomEmoji, schema::custom_emojis, PgPool};
use scoped_futures::ScopedFutureExt;
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

use super::attachment::{AttachmentService, Upload};

const ALLOWED_FILETYPES: &[mime::Name<'_>] = &[mime::IMAGE];

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_allowed_filetype(value: &str, _ctx: &()) -> garde::Result {
    let content_type: mime::Mime = value
        .parse()
        .map_err(|err: mime::FromStrError| garde::Error::new(err.to_string()))?;

    if !ALLOWED_FILETYPES.contains(&content_type.type_()) {
        return Err(garde::Error::new("Invalid file type"));
    }

    Ok(())
}

#[derive(TypedBuilder, Validate)]
pub struct EmojiUpload<S> {
    #[garde(custom(is_allowed_filetype))]
    content_type: String,
    #[garde(length(max = MAX_EMOJI_SHORTCODE_LENGTH))]
    shortcode: Option<String>,
    #[garde(skip)]
    stream: S,
}

#[derive(Clone, TypedBuilder)]
pub struct CustomEmojiService {
    attachment_service: AttachmentService,
    db_pool: PgPool,
}

impl CustomEmojiService {
    pub async fn get_emojis(&self) -> Result<impl Stream<Item = Result<CustomEmoji>> + '_> {
        let query = custom_emojis::table
            .select(CustomEmoji::as_select())
            .order(custom_emojis::id.desc())
            .into_boxed();
        self.db_pool
            .with_connection(|db_conn| {
                async move {
                    Ok::<_, Error>(query.load_stream(db_conn).await?.map_err(Error::from))
                }.scoped()
            })
            .await
            .map_err(Error::from)
    }

    pub async fn add_emoji<S>(&self, upload: EmojiUpload<S>) -> Result<CustomEmoji> {
        upload.validate(&())?;
        Upload::builder().self.attachment_service.upload()
    }
}
