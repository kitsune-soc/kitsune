use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Configuration {
    pub port: u16,
}
