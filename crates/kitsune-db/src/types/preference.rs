use crate::schema::accounts_preferences;
use diesel::{Selectable, prelude::Insertable};
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    Insertable,
    Ord,
    PartialEq,
    PartialOrd,
    Selectable,
    Serialize,
)]
#[diesel(table_name = accounts_preferences)]
#[allow(clippy::struct_excessive_bools)]
pub struct NotificationPreference {
    #[diesel(column_name = "notify_on_follow")]
    pub on_follow: bool,
    #[diesel(column_name = "notify_on_follow_request")]
    pub on_follow_request: bool,
    #[diesel(column_name = "notify_on_repost")]
    pub on_repost: bool,
    #[diesel(column_name = "notify_on_post_update")]
    pub on_post_update: bool,
    #[diesel(column_name = "notify_on_favourite")]
    pub on_favourite: bool,
    #[diesel(column_name = "notify_on_mention")]
    pub on_mention: bool,
}
