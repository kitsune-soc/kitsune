use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod account;
pub mod instance;

pub use self::account::Account;
pub use self::instance::Instance;

#[derive(Deserialize, Serialize)]
pub struct App {
    pub id: Uuid,
    pub name: String,
    pub redirect_uri: String,
    pub client_id: Uuid,
    pub client_secret: String,
}
