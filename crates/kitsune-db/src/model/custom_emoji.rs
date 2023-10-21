use diesel::{Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

use crate::schema::custom_emojis;

#[derive(Clone, Deserialize, Serialize, Identifiable, Insertable, Selectable, Queryable)]
#[diesel(table_name = custom_emojis)]
pub struct CustomEmoji {
    pub id: Uuid,
    pub shortcode: String,
    pub domain: Option<String>,
    pub remote_id: Option<String>,
    pub media_attachment_id: Uuid,
    pub global: bool,
}
