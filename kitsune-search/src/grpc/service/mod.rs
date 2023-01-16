//!
//! Services provided by the gRPC server
//!

mod index;
mod search;

pub use self::{index::IndexService, search::SearchService};
