use crate::error::Result;
use chrono::Utc;
use kitsune_db::entity::{
    accounts, media_attachments, posts_media_attachments, posts_mentions,
    prelude::{MediaAttachments, PostsMediaAttachments, PostsMentions},
};
use kitsune_type::ap::{object::MediaAttachment, Tag, TagType};
use sea_orm::{ConnectionTrait, EntityTrait, IntoActiveModel};
use uuid::Uuid;

pub mod deliverer;
pub mod fetcher;

pub use self::{deliverer::Deliverer, fetcher::Fetcher};

pub async fn handle_attachments<C>(
    db_conn: &C,
    author: &accounts::Model,
    post_id: Uuid,
    attachments: Vec<MediaAttachment>,
) -> Result<()>
where
    C: ConnectionTrait,
{
    if attachments.is_empty() {
        return Ok(());
    }
    let attachment_ids: Vec<Uuid> = (0..attachments.len()).map(|_| Uuid::now_v7()).collect();

    MediaAttachments::insert_many(
        attachments
            .into_iter()
            .zip(attachment_ids.iter().copied())
            .map(|(attachment, attachment_id)| {
                media_attachments::Model {
                    id: attachment_id,
                    account_id: author.id,
                    content_type: attachment.media_type,
                    description: attachment.name,
                    blurhash: attachment.blurhash,
                    file_path: None,
                    remote_url: Some(attachment.url),
                    created_at: Utc::now().into(),
                    updated_at: Utc::now().into(),
                }
                .into_active_model()
            }),
    )
    .exec_with_returning(db_conn)
    .await?;

    PostsMediaAttachments::insert_many(attachment_ids.into_iter().map(|attachment_id| {
        posts_media_attachments::Model {
            post_id,
            media_attachment_id: attachment_id,
        }
        .into_active_model()
    }))
    .exec(db_conn)
    .await?;

    Ok(())
}

pub async fn handle_mentions<'a, C>(
    db_conn: &C,
    author: &accounts::Model,
    post_id: Uuid,
    mentions: &[Tag],
) -> Result<()>
where
    C: ConnectionTrait,
{
    let mention_iter = mentions
        .iter()
        .filter(|mention| mention.r#type == TagType::Mention);

    if mention_iter.clone().count() == 0 {
        return Ok(());
    }

    PostsMentions::insert_many(mention_iter.map(|mention| {
        posts_mentions::Model {
            post_id,
            account_id: author.id,
            mention_text: mention.name.clone(),
        }
        .into_active_model()
    }))
    .exec(db_conn)
    .await?;

    Ok(())
}
