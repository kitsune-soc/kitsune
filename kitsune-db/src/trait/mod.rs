//!
//! Custom traits for database interaction
//!
//! For example, traits for adding permission checks to queries
//!

mod post_permission_check;

pub use self::post_permission_check::PostPermissionCheckExt;
