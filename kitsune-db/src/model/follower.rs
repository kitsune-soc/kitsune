use crate::schema::accounts_follows;
use diesel::{Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Deserialize, Serialize, Identifiable, Queryable)]
#[diesel(table_name = accounts_follows)]
pub struct Follow {
    pub id: Uuid,
    pub account_id: Uuid,
    pub follower_id: Uuid,
    pub url: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = accounts_follows)]
pub struct NewFollow<'a> {
    pub id: Uuid,
    pub account_id: Uuid,
    pub follower_id: Uuid,
    pub url: &'a str,
    pub created_at: Option<OffsetDateTime>,
}
