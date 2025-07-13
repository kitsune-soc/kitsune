use crate::{json::Json, schema::link_previews};
use diesel::prelude::AsChangeset;
use iso8601_timestamp::Timestamp;

#[derive(AsChangeset)]
#[diesel(table_name = link_previews)]
pub struct ConflictLinkPreview<T> {
    pub embed_data: Json<T>,
    pub expires_at: Timestamp,
}
