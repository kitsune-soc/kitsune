use crate::{json::Json, schema::link_previews};
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Identifiable, Queryable, Selectable)]
#[diesel(
    primary_key(url),
    table_name = link_previews,
)]
pub struct LinkPreview<T> {
    pub url: String,
    pub embed_data: Json<T>,
    pub expires_at: Timestamp,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = link_previews)]
pub struct NewLinkPreview<'a, T> {
    pub url: &'a str,
    pub embed_data: Json<T>,
    pub expires_at: Timestamp,
}

#[derive(AsChangeset, Clone)]
#[diesel(table_name = link_previews)]
pub struct ConflictLinkPreviewChangeset<T> {
    pub embed_data: Json<T>,
    pub expires_at: Timestamp,
}
