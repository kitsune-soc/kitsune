use crate::schema::accounts_follows;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Clone, Deserialize, Serialize, Identifiable, Selectable, Queryable)]
#[diesel(table_name = accounts_follows)]
pub struct Follow {
    pub id: Uuid,
    pub account_id: Uuid,
    pub follower_id: Uuid,
    pub approved_at: Option<Timestamp>,
    pub url: String,
    pub notify: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = accounts_follows)]
pub struct NewFollow<'a> {
    pub id: Uuid,
    pub account_id: Uuid,
    pub follower_id: Uuid,
    pub approved_at: Option<Timestamp>,
    pub url: &'a str,
    pub created_at: Option<Timestamp>,
}
