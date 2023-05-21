use crate::schema::users;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use time::OffsetDateTime;
use uuid::Uuid;

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
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
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
}
