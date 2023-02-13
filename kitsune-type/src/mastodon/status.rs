use super::{Account, MediaAttachment};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct Context {
    pub ancestors: VecDeque<Status>,
    pub descendants: Vec<Status>,
}

#[derive(Deserialize, Serialize)]
pub struct Mention {
    pub id: Uuid,
    pub username: String,
    pub url: String,
    pub acct: String,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    #[default]
    Public,
    Unlisted,
    Private,
    Direct,
}

#[derive(Deserialize, Serialize)]
pub struct Status {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
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
}
