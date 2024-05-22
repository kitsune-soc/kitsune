use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct CustomEmoji {
    pub shortcode: String,
    pub url: String,
    pub static_url: String,
    pub visible_in_picker: bool,
    pub category: Option<String>,
}
