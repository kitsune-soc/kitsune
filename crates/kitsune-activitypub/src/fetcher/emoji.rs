use super::Fetcher;
use crate::error::{Error, Result};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use iso8601_timestamp::Timestamp;
use kitsune_db::{
    model::{
        custom_emoji::CustomEmoji,
        media_attachment::{MediaAttachment, NewMediaAttachment},
    },
    schema::{custom_emojis, media_attachments},
};
use kitsune_type::ap::emoji::Emoji;
use scoped_futures::ScopedFutureExt;
use speedy_uuid::Uuid;
use url::Url;

impl Fetcher {
    pub(crate) async fn fetch_emoji(&self, url: &str) -> Result<Option<CustomEmoji>> {
        let existing_emoji = self
            .db_pool
            .with_connection(|db_conn| {
                async move {
                    custom_emojis::table
                        .filter(custom_emojis::remote_id.eq(url))
                        .select(CustomEmoji::as_select())
                        .first(db_conn)
                        .await
                        .optional()
                }
                .scoped()
            })
            .await?;

        if let Some(emoji) = existing_emoji {
            return Ok(Some(emoji));
        }

        let mut url = Url::parse(url)?;
        let Some(emoji) = self.fetch_ap_resource::<_, Emoji>(url.clone()).await? else {
            return Ok(None);
        };

        let mut domain = url.host_str().ok_or(Error::MissingHost)?;

        if emoji.id != url.as_str() {
            url = Url::parse(&emoji.id)?;
            domain = url.host_str().ok_or(Error::MissingHost)?;
        }

        let content_type = emoji
            .icon
            .media_type
            .as_deref()
            .or_else(|| mime_guess::from_path(&emoji.icon.url).first_raw())
            .ok_or(Error::InvalidDocument)?;

        let name_pure = emoji.name.replace(':', "");

        let emoji: CustomEmoji = self
            .db_pool
            .with_transaction(|tx| {
                async move {
                    let media_attachment = diesel::insert_into(media_attachments::table)
                        .values(NewMediaAttachment {
                            id: Uuid::now_v7(),
                            account_id: None,
                            content_type,
                            description: None,
                            blurhash: None,
                            file_path: None,
                            remote_url: Some(&emoji.icon.url),
                        })
                        .returning(MediaAttachment::as_returning())
                        .get_result::<MediaAttachment>(tx)
                        .await?;
                    let emoji = diesel::insert_into(custom_emojis::table)
                        .values(CustomEmoji {
                            id: Uuid::now_v7(),
                            remote_id: emoji.id,
                            shortcode: name_pure.to_string(),
                            domain: Some(domain.to_string()),
                            media_attachment_id: media_attachment.id,
                            endorsed: false,
                            created_at: Timestamp::now_utc(),
                            updated_at: Timestamp::now_utc(),
                        })
                        .returning(CustomEmoji::as_returning())
                        .get_result::<CustomEmoji>(tx)
                        .await?;
                    Ok::<_, Error>(emoji)
                }
                .scoped()
            })
            .await?;

        Ok(Some(emoji))
    }
}
