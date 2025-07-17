use crate::{
    json::Json,
    lang::LanguageIsoCode,
    schema::{
        accounts, accounts_follows, cryptographic_keys, job_context, link_previews,
        media_attachments, posts, users,
    },
    types::{AccountType, Visibility},
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
    pub domain: &'a str,
    pub url: &'a str,
    pub created_at: Option<Timestamp>,
}

#[derive(Insertable)]
#[diesel(table_name = cryptographic_keys)]
pub struct NewCryptographicKey<'a> {
    pub key_id: &'a str,
    pub public_key_der: &'a [u8],
    pub private_key_der: Option<&'a [u8]>,
}

#[derive(Insertable)]
#[diesel(table_name = accounts_follows)]
pub struct NewFollow<'a> {
    pub id: Uuid,
    pub account_id: Uuid,
    pub follower_id: Uuid,
    pub approved_at: Option<Timestamp>,
    pub url: &'a str,
    pub notify: bool,
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
#[diesel(table_name = posts)]
pub struct NewPost<'a> {
    pub id: Uuid,
    pub account_id: Uuid,
    pub in_reply_to_id: Option<Uuid>,
    pub reposted_post_id: Option<Uuid>,
    pub subject: Option<&'a str>,
    pub content: &'a str,
    pub content_source: &'a str,
    pub content_lang: LanguageIsoCode,
    pub link_preview_url: Option<&'a str>,
    pub visibility: Visibility,
    pub is_local: bool,
    pub url: &'a str,
    pub created_at: Option<Timestamp>,
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
