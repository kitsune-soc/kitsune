//!
//! An S3 backed implementation of the [`StorageBackend`] trait
//!

use crate::{Result, StorageBackend};
use async_trait::async_trait;
use futures_util::Stream;

/// S3-backed storage
pub struct Storage {}
