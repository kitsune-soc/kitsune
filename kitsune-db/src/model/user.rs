use crate::schema::users;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use iso8601_timestamp::Timestamp;
use speedy_uuid::Uuid;

#[derive(Clone, Identifiable, Selectable, Queryable)]
pub struct User {
    pub id: Uuid,
    pub account_id: Uuid,
    pub oidc_id: Option<String>,
    pub username: String,
    pub email: String,
    pub password: Option<String>,
    pub domain: String,
    pub private_key: String,

    pub confirmed_at: Option<Timestamp>,
    pub confirmation_token: String,

    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub id: Uuid,
    pub account_id: Uuid,
    pub oidc_id: Option<&'a str>,
    pub username: &'a str,
    pub email: &'a str,
    pub password: Option<&'a str>,
    pub domain: &'a str,
    pub private_key: &'a str,
    pub confirmation_token: &'a str,
}
