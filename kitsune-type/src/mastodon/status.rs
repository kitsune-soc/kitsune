use super::{Account, MediaAttachment, PreviewCard};
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;
use std::collections::VecDeque;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct Context {
    pub ancestors: VecDeque<Status>,
    pub descendants: Vec<Status>,
}

#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct Mention {
    pub id: Uuid,
    pub username: String,
    pub url: String,
    pub acct: String,
}

#[derive(Clone, Copy, Default, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    #[default]
    Public,
    Unlisted,
    Private,
    Direct,
}

#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct Status {
    pub id: Uuid,
    pub created_at: Timestamp,
    pub in_reply_to_id: Option<Uuid>,
    pub in_reply_to_account_id: Option<Uuid>,
    pub sensitive: bool,
    pub spoiler_text: Option<String>,
    pub visibility: Visibility,
    pub uri: String,
    pub url: String,
    pub replies_count: u64,
    pub reblog_count: u64,
    pub favourites_count: u64,
    pub content: String,
    pub account: Account,
    pub media_attachments: Vec<MediaAttachment>,
    pub mentions: Vec<Mention>,
    pub reblog: Option<Box<Status>>,
    pub favourited: bool,
    pub reblogged: bool,
    pub card: Option<PreviewCard>,
}
