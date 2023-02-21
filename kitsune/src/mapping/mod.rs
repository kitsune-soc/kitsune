mod activitypub;

#[cfg(feature = "mastodon-api")]
mod mastodon;

pub use self::activitypub::IntoActivity;
pub use self::activitypub::IntoObject;

#[cfg(feature = "mastodon-api")]
pub use self::mastodon::{MastodonMapper, MastodonMapperBuilder};
