use crate::{
    json::Json,
    lang::LanguageIsoCode,
    schema::{
        accounts, accounts_activitypub, accounts_cryptographic_keys, accounts_follows,
        cryptographic_keys, job_context, jobs, link_previews, media_attachments,
        oauth2_access_tokens, oauth2_applications, oauth2_authorization_codes,
        oauth2_refresh_tokens, posts, posts_favourites, posts_media_attachments, posts_mentions,
        users,
    },
    types::{AccountType, JobState, Visibility},
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
#[diesel(table_name = jobs)]
pub struct NewJob<T> {
    pub id: Uuid,
    pub meta: Json<T>,
    pub state: JobState,
    pub run_at: Timestamp,
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

#[derive(Insertable)]
#[diesel(table_name = posts_favourites)]
pub struct NewFavourite<'a> {
    pub id: Uuid,
    pub account_id: Uuid,
    pub post_id: Uuid,
    pub url: &'a str,
    pub created_at: Option<Timestamp>,
}

#[derive(Insertable)]
#[diesel(table_name = posts_mentions)]
pub struct NewMention<'a> {
    pub post_id: Uuid,
    pub account_id: Uuid,
    pub mention_text: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = posts_media_attachments)]
pub struct NewPostMediaAttachment {
    pub post_id: Uuid,
    pub media_attachment_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = accounts_activitypub)]
pub struct NewAccountsActivitypub<'a> {
    pub account_id: Uuid,
    pub featured_collection_url: Option<&'a str>,
    pub followers_url: Option<&'a str>,
    pub following_url: Option<&'a str>,
    pub inbox_url: Option<&'a str>,
    pub outbox_url: Option<&'a str>,
    pub shared_inbox_url: Option<&'a str>,
    pub key_id: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = accounts_cryptographic_keys)]
pub struct NewAccountsCryptographicKey<'a> {
    pub account_id: Uuid,
    pub key_id: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = oauth2_access_tokens)]
pub struct NewOauth2AccessToken<'a> {
    pub user_id: Option<Uuid>,
    pub application_id: Option<Uuid>,
    pub token: &'a str,
    pub scopes: &'a str,
    pub expires_at: Timestamp,
}

#[derive(Insertable)]
#[diesel(table_name = oauth2_applications)]
pub struct NewOauth2Application<'a> {
    pub id: Uuid,
    pub secret: &'a str,
    pub name: &'a str,
    pub redirect_uri: &'a str,
    pub scopes: &'a str,
    pub website: Option<&'a str>,
}

#[derive(Insertable)]
#[diesel(table_name = oauth2_authorization_codes)]
pub struct NewOauth2AuthorizationCode<'a> {
    pub code: &'a str,
    pub user_id: Uuid,
    pub application_id: Uuid,
    pub scopes: &'a str,
    pub expires_at: Timestamp,
}

#[derive(Insertable)]
#[diesel(table_name = oauth2_refresh_tokens)]
pub struct NewOauth2RefreshToken<'a> {
    pub token: &'a str,
    pub access_token: &'a str,
    pub application_id: Uuid,
}
