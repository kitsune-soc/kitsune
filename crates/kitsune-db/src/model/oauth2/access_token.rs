use super::{super::user::User, application::Application};
use crate::schema::oauth2_access_tokens;
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Associations, Clone, Deserialize, Identifiable, Selectable, Serialize, Queryable)]
#[diesel(
    belongs_to(Application),
    belongs_to(User),
    primary_key(token),
    table_name = oauth2_access_tokens,
)]
pub struct AccessToken {
    pub token: String,
    pub user_id: Option<Uuid>,
    pub application_id: Option<Uuid>,
    pub scopes: String,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = oauth2_access_tokens)]
pub struct NewAccessToken<'a> {
    pub token: &'a str,
    pub user_id: Option<Uuid>,
    pub application_id: Option<Uuid>,
    pub scopes: &'a str,
    pub expires_at: Timestamp,
}
