//!
//! Custom traits for database interaction
//!
//! For example, traits for adding permission checks to queries
//!

mod account_permission_check;
mod post_permission_check;

pub use self::{
    account_permission_check::AccountPermissionCheckExt,
    post_permission_check::{PermissionCheck, PostPermissionCheckExt},
};
