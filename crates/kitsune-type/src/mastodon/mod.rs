use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

pub mod account;
pub mod custom_emoji;
pub mod instance;
pub mod media_attachment;
pub mod notification;
pub mod preview_card;
pub mod relationship;
pub mod search;
pub mod status;

pub use self::account::Account;
pub use self::custom_emoji::CustomEmoji;
pub use self::instance::Instance;
pub use self::media_attachment::MediaAttachment;
pub use self::notification::Notification;
pub use self::preview_card::PreviewCard;
pub use self::search::SearchResult;
pub use self::status::Status;

#[derive(Deserialize, Serialize)]
pub struct App {
    pub id: Uuid,
    pub name: String,
    pub redirect_uri: String,
    pub client_id: Uuid,
    pub client_secret: String,
}
