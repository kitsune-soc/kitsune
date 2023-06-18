use crate::schema::link_previews;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Deserialize, Serialize, Identifiable, Queryable, Selectable)]
#[diesel(
    primary_key(url),
    table_name = link_previews,
)]
pub struct LinkPreview {
    pub url: String,
    pub embed_data: Value,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = link_previews)]
pub struct NewLinkPreview<'a> {
    pub url: &'a str,
    pub embed_data: &'a Value,
}
