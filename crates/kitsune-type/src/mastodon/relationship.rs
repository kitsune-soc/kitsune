use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;
use utoipa::ToSchema;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, ToSchema)]
#[allow(clippy::struct_excessive_bools)]
pub struct Relationship {
    pub id: Uuid,
    pub following: bool,
    pub showing_reblogs: bool,
    pub notifying: bool,
    pub followed_by: bool,
    pub blocking: bool,
    pub blocked_by: bool,
    pub muting: bool,
    pub muting_notifications: bool,
    pub requested: bool,
    pub domain_blocking: bool,
    pub endorsed: bool,
    pub note: String,
}
