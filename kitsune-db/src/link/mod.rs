//!
//! `SeaORM` links defining more complicated relationships, usually spanning across multiple tables
//!

mod favourited_post_author;
mod followers;
mod following;
mod in_reply_to;
mod mentioned_accounts;

pub use self::favourited_post_author::FavouritedPostAuthor;
pub use self::followers::Followers;
pub use self::following::Following;
pub use self::in_reply_to::InReplyTo;
pub use self::mentioned_accounts::MentionedAccounts;
