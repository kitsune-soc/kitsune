use super::{account::Account, post::Post};
use crate::schema::{media_attachments, posts_media_attachments};
use diesel::{Associations, Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Associations, Clone, Deserialize, Identifiable, Serialize, Queryable)]
#[diesel(belongs_to(Account), table_name = media_attachments)]
pub struct MediaAttachment {
    pub id: Uuid,
    pub account_id: Uuid,
    pub content_type: String,
    pub description: Option<String>,
    pub blurhash: Option<String>,
    pub file_path: Option<String>,
    pub remote_url: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = media_attachments)]
pub struct NewMediaAttachment<'a> {
    pub id: Uuid,
    pub account_id: Uuid,
    pub content_type: &'a str,
    pub description: Option<&'a str>,
    pub blurhash: Option<&'a str>,
    pub file_path: Option<&'a str>,
    pub remote_url: Option<&'a str>,
}

#[derive(Associations, Clone, Deserialize, Identifiable, Insertable, Serialize, Queryable)]
#[diesel(
    belongs_to(MediaAttachment),
    belongs_to(Post),
    primary_key(media_attachment_id, post_id),
    table_name = posts_media_attachments,
)]
pub struct PostMediaAttachment {
    pub post_id: Uuid,
    pub media_attachment_id: Uuid,
}

pub type NewPostMediaAttachment = PostMediaAttachment;
