use serde::Deserialize;

#[derive(Deserialize)]
pub struct Configuration {
    pub database_url: String,
    #[serde(default)]
    pub database_use_tls: bool,
}
