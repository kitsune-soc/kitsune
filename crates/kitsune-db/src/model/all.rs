#![allow(clippy::wildcard_imports)]

use crate::{
    json::Json,
    lang::LanguageIsoCode,
    schema::*,
    types::{ActorType, JobState, Visibility},
};
use diesel::{Identifiable, Queryable, Selectable};
use iso8601_timestamp::Timestamp;
use speedy_uuid::Uuid;

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(table_name = accounts)]
pub struct Account {
    pub id: Uuid,
    pub account_type: ActorType,
    pub protocol: i32,
    pub avatar_id: Option<Uuid>,
    pub header_id: Option<Uuid>,
    pub display_name: Option<String>,
    pub note: Option<String>,
    pub username: String,
    pub locked: bool,
    pub local: bool,
    pub domain: String,
    pub url: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(primary_key(account_id))]
#[diesel(table_name = accounts_activitypub)]
pub struct AccountsActivitypub {
    pub account_id: Uuid,
    pub featured_collection_url: Option<String>,
    pub followers_url: Option<String>,
    pub following_url: Option<String>,
    pub inbox_url: Option<String>,
    pub outbox_url: Option<String>,
    pub shared_inbox_url: Option<String>,
    pub key_id: String,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(primary_key(account_id, key_id))]
#[diesel(table_name = accounts_cryptographic_keys)]
pub struct AccountsCryptographicKey {
    pub account_id: Uuid,
    pub key_id: String,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(table_name = accounts_follows)]
pub struct AccountsFollow {
    pub id: Uuid,
    pub account_id: Uuid,
    pub follower_id: Uuid,
    pub approved_at: Option<Timestamp>,
    pub url: String,
    pub notify: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(primary_key(account_id))]
#[diesel(table_name = accounts_preferences)]
pub struct AccountsPreference {
    pub account_id: Uuid,
    pub notify_on_follow: bool,
    pub notify_on_follow_request: bool,
    pub notify_on_repost: bool,
    pub notify_on_post_update: bool,
    pub notify_on_favourite: bool,
    pub notify_on_mention: bool,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(primary_key(key_id))]
#[diesel(table_name = cryptographic_keys)]
pub struct CryptographicKey {
    pub key_id: String,
    pub public_key_der: Vec<u8>,
    pub private_key_der: Option<Vec<u8>>,
    pub created_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(table_name = custom_emojis)]
pub struct CustomEmoji {
    pub id: Uuid,
    pub shortcode: String,
    pub domain: Option<String>,
    pub remote_id: String,
    pub media_attachment_id: Uuid,
    pub endorsed: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(primary_key(domain))]
#[diesel(table_name = domains)]
pub struct Domain {
    pub domain: String,
    pub owner_id: Option<Uuid>,
    pub challenge_value: Option<String>,
    pub globally_available: bool,
    pub verified_at: Option<Timestamp>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(table_name = job_context)]
pub struct JobContext<T> {
    pub id: Uuid,
    pub context: Json<T>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(table_name = jobs)]
pub struct Job<T> {
    pub id: Uuid,
    pub meta: Json<T>,
    pub state: JobState,
    pub fail_count: i32,
    pub run_at: Timestamp,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(primary_key(url))]
#[diesel(table_name = link_previews)]
pub struct LinkPreview<T> {
    pub url: String,
    pub embed_data: Json<T>,
    pub expires_at: Timestamp,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(table_name = media_attachments)]
pub struct MediaAttachment {
    pub id: Uuid,
    pub account_id: Option<Uuid>,
    pub content_type: String,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub is_sensitive: bool,
    pub remote_url: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(table_name = notifications)]
pub struct Notification {
    pub id: Uuid,
    pub receiving_account_id: Uuid,
    pub triggering_account_id: Option<Uuid>,
    pub post_id: Option<Uuid>,
    pub notification_type: i16,
    pub created_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(primary_key(token))]
#[diesel(table_name = oauth2_access_tokens)]
pub struct Oauth2AccessToken {
    pub token: String,
    pub user_id: Option<Uuid>,
    pub application_id: Option<Uuid>,
    pub scopes: String,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(table_name = oauth2_applications)]
pub struct Oauth2Application {
    pub id: Uuid,
    pub name: String,
    pub secret: String,
    pub scopes: String,
    pub redirect_uri: String,
    pub website: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(primary_key(code))]
#[diesel(table_name = oauth2_authorization_codes)]
pub struct Oauth2AuthorizationCode {
    pub code: String,
    pub application_id: Uuid,
    pub user_id: Uuid,
    pub scopes: String,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(primary_key(token))]
#[diesel(table_name = oauth2_refresh_tokens)]
pub struct Oauth2RefreshToken {
    pub token: String,
    pub access_token: String,
    pub application_id: Uuid,
    pub created_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: Uuid,
    pub account_id: Uuid,
    pub in_reply_to_id: Option<Uuid>,
    pub reposted_post_id: Option<Uuid>,
    pub subject: Option<String>,
    pub content: String,
    pub content_source: String,
    pub content_lang: LanguageIsoCode,
    pub link_preview_url: Option<String>,
    pub visibility: Visibility,
    pub is_local: bool,
    pub url: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(primary_key(post_id, custom_emoji_id))]
#[diesel(table_name = posts_custom_emojis)]
pub struct PostsCustomEmoji {
    pub post_id: Uuid,
    pub custom_emoji_id: Uuid,
    pub emoji_text: String,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(table_name = posts_favourites)]
pub struct PostsFavourite {
    pub id: Uuid,
    pub account_id: Uuid,
    pub post_id: Uuid,
    pub url: String,
    pub created_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(primary_key(post_id, media_attachment_id))]
#[diesel(table_name = posts_media_attachments)]
pub struct PostsMediaAttachment {
    pub post_id: Uuid,
    pub media_attachment_id: Uuid,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(primary_key(post_id, account_id))]
#[diesel(table_name = posts_mentions)]
pub struct PostsMention {
    pub post_id: Uuid,
    pub account_id: Uuid,
    pub mention_text: String,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(table_name = roles)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub capabilities: Vec<Option<i32>>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub oidc_id: Option<String>,
    pub username: String,
    pub email: String,
    pub password: Option<String>,
    pub confirmed_at: Option<Timestamp>,
    pub confirmation_token: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(primary_key(user_id, account_id))]
#[diesel(table_name = users_accounts)]
pub struct UsersAccount {
    pub user_id: Uuid,
    pub account_id: Uuid,
}

#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(primary_key(user_id, role_id))]
#[diesel(table_name = users_roles)]
pub struct UsersRole {
    pub user_id: Uuid,
    pub role_id: Uuid,
}
