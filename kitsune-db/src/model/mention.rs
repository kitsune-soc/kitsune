use super::{account::Account, post::Post};
use crate::schema::posts_mentions;
use diesel::{Associations, Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Associations, Clone, Deserialize, Identifiable, Queryable, Serialize)]
#[diesel(
    belongs_to(Account),
    belongs_to(Post),
    primary_key(account_id, post_id),
    table_name = posts_mentions,
)]
pub struct Mention {
    pub post_id: Uuid,
    pub account_id: Uuid,
    pub mention_text: String,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = posts_mentions)]
pub struct NewMention<'a> {
    pub post_id: Uuid,
    pub account_id: Uuid,
    pub mention_text: &'a str,
}
