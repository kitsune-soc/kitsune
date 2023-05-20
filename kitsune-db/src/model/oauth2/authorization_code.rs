use super::{super::user::User, application::Application};
use crate::schema::oauth2_authorization_codes;
use diesel::{Associations, Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

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
    pub created_at: OffsetDateTime,
    pub expired_at: OffsetDateTime,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = oauth2_authorization_codes)]
pub struct NewAuthorizationCode<'a> {
    pub code: &'a str,
    pub application_id: Uuid,
    pub user_id: Uuid,
    pub expired_at: OffsetDateTime,
}
