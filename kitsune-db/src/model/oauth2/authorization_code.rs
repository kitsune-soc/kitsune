use super::{super::user::User, application::Application};
use crate::schema::oauth2_authorization_codes;
use diesel::{Associations, Identifiable, Insertable, Queryable};
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Associations, Clone, Deserialize, Identifiable, Serialize, Queryable)]
#[diesel(
    belongs_to(Application),
    belongs_to(User),
    primary_key(code),
    table_name = oauth2_authorization_codes,
)]
pub struct AuthorizationCode {
    pub code: String,
    pub application_id: Uuid,
    pub user_id: Uuid,
    pub scopes: String,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = oauth2_authorization_codes)]
pub struct NewAuthorizationCode<'a> {
    pub code: &'a str,
    pub application_id: Uuid,
    pub user_id: Uuid,
    pub scopes: &'a str,
    pub expires_at: Timestamp,
}
