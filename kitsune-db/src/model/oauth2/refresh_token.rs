use super::{access_token::AccessToken, application::Application};
use crate::schema::oauth2_refresh_tokens;
use diesel::{Associations, Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Associations, Clone, Deserialize, Identifiable, Serialize, Queryable)]
#[diesel(
    belongs_to(
        AccessToken,
        foreign_key = access_token,
    ),
    belongs_to(Application),
    primary_key(token),
    table_name = oauth2_refresh_tokens,
)]
pub struct RefreshToken {
    pub token: String,
    pub access_token: String,
    pub application_id: Uuid,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = oauth2_refresh_tokens)]
pub struct NewRefreshToken<'a> {
    pub token: &'a str,
    pub access_token: &'a str,
    pub application_id: Uuid,
}
