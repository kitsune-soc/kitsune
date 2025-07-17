use crate::{
    json::Json,
    lang::LanguageIsoCode,
    schema::{accounts, link_previews, media_attachments, posts},
};
use diesel::prelude::AsChangeset;
use iso8601_timestamp::Timestamp;
use speedy_uuid::Uuid;

#[derive(AsChangeset)]
#[diesel(table_name = link_previews)]
pub struct ConflictLinkPreview<T> {
    pub embed_data: Json<T>,
    pub expires_at: Timestamp,
}

#[derive(AsChangeset)]
#[diesel(table_name = posts)]
pub struct PartialPostChangeset<'a> {
    pub id: Uuid,
    pub subject: Option<&'a str>,
    pub content: Option<&'a str>,
    pub content_source: Option<&'a str>,
    pub content_lang: Option<LanguageIsoCode>,
    pub link_preview_url: Option<&'a str>,
    pub updated_at: Timestamp,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = accounts)]
pub struct UpdateAccount<'a> {
    pub display_name: Option<&'a str>,
    pub note: Option<&'a str>,
    pub avatar_id: Option<Uuid>,
    pub header_id: Option<Uuid>,
    pub locked: Option<bool>,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = media_attachments)]
pub struct UpdateMediaAttachment<'a> {
    pub description: Option<&'a str>,
}
