mod activitypub;

#[cfg(feature = "mastodon-api")]
mod mastodon;

pub use self::activitypub::IntoActivityPub;

#[cfg(feature = "mastodon-api")]
pub use self::mastodon::IntoMastodon;
