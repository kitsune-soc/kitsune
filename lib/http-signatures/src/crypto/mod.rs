//!
//! Common cryptographic operations
//!

mod sign;
mod verify;

pub mod parse;

pub use self::sign::{SigningKey, sign};
pub use self::verify::{VerifyError, verify};
