use crate::{
    json::Json,
    schema::{accounts, job_context, link_previews, media_attachments, users},
    types::{AccountType, Protocol},
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
    pub protocol: Protocol,
    pub avatar_id: Option<Uuid>,
    pub header_id: Option<Uuid>,
    pub display_name: Option<&'a str>,
    pub note: Option<&'a str>,
    pub username: &'a str,
    pub locked: bool,
    pub local: bool,
    pub domain: &'a str,
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

#[derive(Insertable)]
#[diesel(table_name = media_attachments)]
pub struct NewMediaAttachment<'a> {
    pub id: Uuid,
    pub content_type: &'a str,
    pub account_id: Option<Uuid>,
    pub description: Option<&'a str>,
    pub file_path: Option<&'a str>,
    pub remote_url: Option<&'a str>,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub id: Uuid,
    pub oidc_id: Option<&'a str>,
    pub username: &'a str,
    pub email: &'a str,
    pub password: Option<&'a str>,
    pub confirmation_token: &'a str,
}
