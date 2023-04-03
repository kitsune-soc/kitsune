use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub mod account;
pub mod instance;
pub mod media_attachment;
pub mod relationship;
pub mod search;
pub mod status;

pub use self::account::Account;
pub use self::instance::Instance;
pub use self::media_attachment::MediaAttachment;
pub use self::search::SearchResult;
pub use self::status::Status;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct App {
    pub id: Uuid,
    pub name: String,
    pub redirect_uri: String,
    pub client_id: Uuid,
    pub client_secret: String,
}
