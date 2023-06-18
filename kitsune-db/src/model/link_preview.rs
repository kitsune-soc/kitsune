use crate::{json::Json, schema::link_previews};
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Clone, Deserialize, Serialize, Identifiable, Queryable, Selectable)]
#[diesel(
    primary_key(url),
    table_name = link_previews,
)]
pub struct LinkPreview<T> {
    pub url: String,
    pub embed_data: Json<T>,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = link_previews)]
pub struct NewLinkPreview<'a, T> {
    pub url: &'a str,
    pub embed_data: Json<T>,
    pub expires_at: OffsetDateTime,
}

#[derive(AsChangeset, Clone)]
#[diesel(table_name = link_previews)]
pub struct ConflictLinkPreviewChangeset<T> {
    pub embed_data: Json<T>,
    pub expires_at: OffsetDateTime,
}
