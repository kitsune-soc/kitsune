use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod account;
pub mod instance;
pub mod media_attachment;
pub mod status;

pub use self::account::Account;
pub use self::instance::Instance;
pub use self::media_attachment::MediaAttachment;
pub use self::status::Status;

#[derive(Deserialize, Serialize)]
pub struct App {
    pub id: Uuid,
    pub name: String,
    pub redirect_uri: String,
    pub client_id: Uuid,
    pub client_secret: String,
}
