use crate::schema::oauth2_applications;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Deserialize, Identifiable, Selectable, Serialize, Queryable)]
#[diesel(table_name = oauth2_applications)]
pub struct Application {
    pub id: Uuid,
    pub name: String,
    pub secret: String,
    pub scopes: String,
    pub redirect_uri: String,
    pub website: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = oauth2_applications)]
pub struct NewApplication<'a> {
    pub id: Uuid,
    pub name: &'a str,
    pub secret: &'a str,
    pub scopes: &'a str,
    pub redirect_uri: &'a str,
    pub website: Option<&'a str>,
}
