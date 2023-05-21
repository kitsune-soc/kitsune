use crate::error::Result;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use kitsune_db::{
    model::{
        account::Account,
        media_attachment::{NewMediaAttachment, NewPostMediaAttachment},
        mention::NewMention,
    },
    schema::{media_attachments, posts_media_attachments, posts_mentions},
};
use kitsune_type::ap::{object::MediaAttachment, Tag, TagType};
use uuid::Uuid;

pub mod deliverer;
pub mod fetcher;

pub use self::{deliverer::Deliverer, fetcher::Fetcher};

pub async fn handle_attachments(
    db_conn: &mut AsyncPgConnection,
    author: &Account,
    post_id: Uuid,
    attachments: Vec<MediaAttachment>,
) -> Result<()> {
    if attachments.is_empty() {
        return Ok(());
    }
    let attachment_ids: Vec<Uuid> = (0..attachments.len()).map(|_| Uuid::now_v7()).collect();

    diesel::insert_into(media_attachments::table)
        .values(
            attachments
                .iter()
                .zip(attachment_ids.iter().copied())
                .map(|(attachment, attachment_id)| NewMediaAttachment {
                    id: attachment_id,
                    account_id: author.id,
                    content_type: attachment.media_type.as_str(),
                    description: attachment.name.as_deref(),
                    blurhash: attachment.blurhash.as_deref(),
                    file_path: None,
                    remote_url: Some(attachment.url.as_str()),
                })
                .collect::<Vec<NewMediaAttachment<'_>>>(),
        )
        .execute(db_conn)
        .await?;

    diesel::insert_into(posts_media_attachments::table)
        .values(
            attachment_ids
                .into_iter()
                .map(|attachment_id| NewPostMediaAttachment {
                    post_id,
                    media_attachment_id: attachment_id,
                })
                .collect::<Vec<NewPostMediaAttachment>>(),
        )
        .execute(db_conn)
        .await?;

    Ok(())
}

pub async fn handle_mentions(
    db_conn: &mut AsyncPgConnection,
    author: &Account,
    post_id: Uuid,
    mentions: &[Tag],
) -> Result<()> {
    let mention_iter = mentions
        .iter()
        .filter(|mention| mention.r#type == TagType::Mention);

    if mention_iter.clone().count() == 0 {
        return Ok(());
    }

    diesel::insert_into(posts_mentions::table)
        .values(
            mention_iter
                .map(|mention| NewMention {
                    post_id,
                    account_id: author.id,
                    mention_text: mention.name.as_str(),
                })
                .collect::<Vec<NewMention<'_>>>(),
        )
        .execute(db_conn)
        .await?;

    Ok(())
}
