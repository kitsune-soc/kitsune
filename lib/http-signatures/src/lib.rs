//!
//! Implementation of HTTP signature standards with custom opinionated defaults to ensure authenticity
//!
//! ## Standards
//!
//! - [x] [Cavage HTTP signatures](https://datatracker.ietf.org/doc/html/draft-cavage-http-signatures-12)
//! - [ ] [RFC 9421 signatures](https://datatracker.ietf.org/doc/html/rfc9421)
//!

#![deny(missing_docs)]

use http::HeaderName;

pub mod cavage;
pub mod crypto;

/// Boxed error with `Send` and `Sync` bounds
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Header name for the signature
pub static SIGNATURE_HEADER: HeaderName = HeaderName::from_static("signature");
