use crate::{
    json::Json,
    schema::{accounts, job_context, link_previews},
    types::AccountType,
};
use diesel::prelude::Insertable;
use iso8601_timestamp::Timestamp;
use speedy_uuid::Uuid;

mod notification;

pub use self::notification::NewNotification;

#[derive(Insertable)]
#[diesel(table_name = accounts)]
pub struct NewAccount<'a> {
    pub id: Uuid,
    pub account_type: AccountType,
    pub avatar_id: Option<Uuid>,
    pub header_id: Option<Uuid>,
    pub display_name: Option<&'a str>,
    pub note: Option<&'a str>,
    pub username: &'a str,
    pub locked: bool,
    pub local: bool,
    pub url: &'a str,
    pub created_at: Option<Timestamp>,
}

#[derive(Insertable)]
#[diesel(table_name = job_context)]
pub struct NewJobContext<T> {
    pub id: Uuid,
    pub context: Json<T>,
}

#[derive(Insertable)]
#[diesel(table_name = link_previews)]
pub struct NewLinkPreview<'a, T> {
    pub url: &'a str,
    pub embed_data: Json<T>,
    pub expires_at: Timestamp,
}
