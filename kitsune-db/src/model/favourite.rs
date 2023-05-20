use super::{account::Account, post::Post};
use crate::schema::posts_favourites;
use diesel::{Associations, Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Associations, Clone, Deserialize, Identifiable, Serialize, Queryable)]
#[diesel(
    belongs_to(Account),
    belongs_to(Post),
    table_name = posts_favourites,
)]
pub struct Favourite {
    pub id: Uuid,
    pub account_id: Uuid,
    pub post_id: Uuid,
    pub url: String,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = posts_favourites)]
pub struct NewFavourite {
    pub id: Uuid,
    pub account_id: Uuid,
    pub post_id: Uuid,
    pub url: String,
    pub created_at: Option<OffsetDateTime>,
}
