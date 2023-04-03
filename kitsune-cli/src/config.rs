use serde::Deserialize;

#[derive(Deserialize)]
pub struct Configuration {
    pub database_url: String,
}
