//!
//! Tonic types generated from protobuf definitions by `tonic-build`
//!

#![forbid(rust_2018_idioms, unsafe_code)]
#![deny(missing_docs)]

/// Common types
pub mod common {
    tonic::include_proto!("kitsune.common");
}

/// Indexing types and services
pub mod index {
    tonic::include_proto!("kitsune.index");
}

/// Search types and services
pub mod search {
    tonic::include_proto!("kitsune.search");
}
