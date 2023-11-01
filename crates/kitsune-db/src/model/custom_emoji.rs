use super::post::Post;
use crate::schema::posts_custom_emojis;
use diesel::{AsChangeset, Associations, Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

use crate::schema::custom_emojis;

#[derive(Clone, Deserialize, Serialize, Identifiable, Insertable, Selectable, Queryable)]
#[diesel(table_name = custom_emojis)]
pub struct CustomEmoji {
    pub id: Uuid,
    pub shortcode: String,
    pub domain: Option<String>,
    pub remote_id: String,
    pub media_attachment_id: Uuid,
    pub endorsed: bool,
}

#[derive(AsChangeset)]
#[diesel(table_name = custom_emojis)]
pub struct CustomEmojiConflictChangeset {
    pub media_attachment_id: Uuid,
}

#[derive(
    Associations, Clone, Deserialize, Identifiable, Insertable, Queryable, Selectable, Serialize,
)]
#[diesel(
    belongs_to(CustomEmoji),
    belongs_to(Post),
    primary_key(custom_emoji_id, post_id),
    table_name = posts_custom_emojis,
)]
pub struct PostCustomEmoji {
    pub post_id: Uuid,
    pub custom_emoji_id: Uuid,
    pub emoji_text: String,
}
