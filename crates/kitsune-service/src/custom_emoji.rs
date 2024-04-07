use super::attachment::{AttachmentService, Upload};
use bytes::Bytes;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    OptionalExtension, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use futures_util::{Stream, TryStreamExt};
use garde::Validate;
use iso8601_timestamp::Timestamp;
use kitsune_core::consts::MAX_EMOJI_SHORTCODE_LENGTH;
use kitsune_db::{
    model::{custom_emoji::CustomEmoji, media_attachment::MediaAttachment},
    schema::{custom_emojis, media_attachments, posts, posts_custom_emojis},
    with_connection, PgPool,
};
use kitsune_error::{Error, Result};
use kitsune_url::UrlService;
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

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

#[derive(TypedBuilder)]
pub struct GetEmojiList {
    #[builder(default)]
    fetching_account_id: Option<Uuid>,
    #[builder(default = 5000)]
    limit: i64,
}

#[derive(TypedBuilder, Validate)]
pub struct EmojiUpload<S> {
    #[garde(custom(is_allowed_filetype))]
    content_type: String,
    #[garde(length(chars, max = MAX_EMOJI_SHORTCODE_LENGTH))]
    #[garde(pattern("^([a-zA-Z0-9]_?)*[a-zA-Z0-9]$"))]
    shortcode: String,
    #[garde(skip)]
    stream: S,
}

#[derive(Clone, TypedBuilder)]
pub struct CustomEmojiService {
    attachment_service: AttachmentService,
    db_pool: PgPool,
    url_service: UrlService,
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

        with_connection!(self.db_pool, |db_conn| {
            query.first(db_conn).await.optional()
        })
        .map_err(Error::from)
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<CustomEmoji> {
        let query = custom_emojis::table
            .find(id)
            .select(CustomEmoji::as_select());

        with_connection!(self.db_pool, |db_conn| { query.get_result(db_conn).await })
            .map_err(Error::from)
    }

    pub async fn get_list(
        &self,
        get_emoji_list: GetEmojiList,
    ) -> Result<impl Stream<Item = Result<(CustomEmoji, MediaAttachment, Option<Timestamp>)>> + '_>
    {
        let query = custom_emojis::table
            .left_join(
                posts_custom_emojis::table.inner_join(
                    posts::table.on(posts::account_id
                        .nullable()
                        .eq(get_emoji_list.fetching_account_id)),
                ),
            )
            .inner_join(media_attachments::table)
            .filter(
                posts::account_id.is_null().or(posts::account_id
                    .nullable()
                    .eq(get_emoji_list.fetching_account_id)),
            )
            .filter(
                custom_emojis::endorsed
                    .eq(true)
                    .or(custom_emojis::domain.is_null())
                    .or(posts::created_at.is_not_null()),
            )
            .distinct_on(custom_emojis::id)
            .select((
                CustomEmoji::as_select(),
                MediaAttachment::as_select(),
                posts::created_at.nullable(),
            ))
            .limit(get_emoji_list.limit);

        with_connection!(self.db_pool, |db_conn| {
            Ok::<_, Error>(query.load_stream(db_conn).await?.map_err(Error::from))
        })
    }

    pub async fn add_emoji<S>(&self, emoji_upload: EmojiUpload<S>) -> Result<CustomEmoji>
    where
        S: Stream<Item = Result<Bytes>> + Send + Sync + 'static,
    {
        emoji_upload.validate(&())?;

        let attachment_upload = Upload::builder()
            .content_type(emoji_upload.content_type)
            .stream(emoji_upload.stream)
            .build()
            .unwrap();

        let attachment = self.attachment_service.upload(attachment_upload).await?;

        let id = Uuid::now_v7();
        let remote_id = self.url_service.custom_emoji_url(id);

        let custom_emoji = with_connection!(self.db_pool, |db_conn| {
            diesel::insert_into(custom_emojis::table)
                .values(CustomEmoji {
                    id,
                    remote_id,
                    shortcode: emoji_upload.shortcode,
                    domain: None,
                    media_attachment_id: attachment.id,
                    endorsed: false,
                    created_at: Timestamp::now_utc(),
                    updated_at: Timestamp::now_utc(),
                })
                .get_result(db_conn)
                .await
        })?;

        Ok(custom_emoji)
    }
}
