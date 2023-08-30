use diesel::{Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

use crate::schema::accounts_preferences;

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Deserialize, Serialize, Identifiable, Insertable, Selectable, Queryable)]
#[diesel(table_name = accounts_preferences)]
#[diesel(primary_key(account_id))]
pub struct Preferences {
    pub account_id: Uuid,
    pub notify_on_follow: bool,
    pub notify_on_follow_request: bool,
    pub notify_on_repost: bool,
    pub notify_on_favourite: bool,
    pub notify_on_mention: bool,
    pub notify_on_repost_update: bool,
}
