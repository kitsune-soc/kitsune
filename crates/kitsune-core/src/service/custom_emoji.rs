use crate::{
    consts::MAX_EMOJI_SHORTCODE_LENGTH,
    error::{BoxError, Error, Result},
};

use bytes::Bytes;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::{Stream, TryStreamExt};
use garde::Validate;
use kitsune_db::{
    model::custom_emoji::CustomEmoji,
    schema::{custom_emojis, media_attachments},
    PgPool,
};
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

#[derive(TypedBuilder)]
pub struct GetEmoji<'a> {
    shortcode: &'a str,
    #[builder(default)]
    domain: Option<&'a str>,
}

#[derive(TypedBuilder, Validate)]
pub struct EmojiUpload<S> {
    #[garde(custom(is_allowed_filetype))]
    content_type: String,
    #[garde(length(max = MAX_EMOJI_SHORTCODE_LENGTH))]
    shortcode: String,
    #[garde(skip)]
    stream: S,
}

#[derive(Clone, TypedBuilder)]
pub struct CustomEmojiService {
    attachment_service: AttachmentService,
    db_pool: PgPool,
}

impl CustomEmojiService {
    pub async fn get(&self, get_emoji: GetEmoji<'_>) -> Result<Option<CustomEmoji>> {
        let mut query = custom_emojis::table
            .filter(custom_emojis::shortcode.eq(get_emoji.shortcode))
            .inner_join(media_attachments::table)
            .select(CustomEmoji::as_select())
            .into_boxed();

        if let Some(domain) = get_emoji.domain {
            query = query.filter(custom_emojis::domain.eq(domain));
        }

        self.db_pool
            .with_connection(|db_conn| {
                async move { query.first(db_conn).await.optional() }.scoped()
            })
            .await
            .map_err(Error::from)
    }

    pub async fn get_all(&self) -> Result<impl Stream<Item = Result<CustomEmoji>> + '_> {
        let query = custom_emojis::table
            .select(CustomEmoji::as_select())
            .order(custom_emojis::id.desc())
            .into_boxed();
        self.db_pool
            .with_connection(|db_conn| {
                async move { Ok::<_, Error>(query.load_stream(db_conn).await?.map_err(Error::from)) }
                    .scoped()
            })
            .await
            .map_err(Error::from)
    }

    pub async fn add_emoji<S>(&self, emoji_upload: EmojiUpload<S>) -> Result<CustomEmoji>
    where
        S: Stream<Item = Result<Bytes, BoxError>> + Send + 'static,
    {
        emoji_upload.validate(&())?;

        let attachment_upload = Upload::builder()
            .content_type(emoji_upload.content_type)
            .stream(emoji_upload.stream)
            .build()
            .unwrap();

        let attachment = self.attachment_service.upload(attachment_upload).await?;

        let custom_emoji = self
            .db_pool
            .with_connection(|db_conn| {
                diesel::insert_into(custom_emojis::table)
                    .values(CustomEmoji {
                        id: Uuid::now_v7(),
                        remote_id: None,
                        shortcode: emoji_upload.shortcode,
                        domain: None,
                        media_attachment_id: attachment.id,
                        endorsed: false,
                    })
                    .get_result(db_conn)
                    .scoped()
            })
            .await?;

        Ok(custom_emoji)
    }
}
