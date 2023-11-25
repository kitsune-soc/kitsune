use std::error::Error as StdError;

pub type BoxError = Box<dyn StdError + Send + Sync>;
