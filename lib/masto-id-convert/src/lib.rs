#![doc = include_str!("../README.md")]
#![forbid(missing_docs, rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt;
use nanorand::{Rng, WyRand};
use uuid::Uuid;

/// Combined error type
#[derive(Debug)]
pub enum Error {
    /// Number parsing error
    Lexical(lexical::Error),
}

impl From<lexical::Error> for Error {
    fn from(value: lexical::Error) -> Self {
        Self::Lexical(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// Process a Mastodon snowflake in its u64 representation into a UUID v7 identifier
#[inline]
#[must_use]
pub fn process_u64(masto_id: u64) -> Uuid {
    let timestamp_ms = (masto_id >> 16) & 0xFF_FF_FF_FF_FF_FF;
    let sequence_data = masto_id & 0xFF_FF;
    let mut wyrand = WyRand::new_seed(sequence_data);

    let mut rand_data = [0; 10];
    wyrand.fill_bytes(&mut rand_data);

    let mut raw_uuid = [0; 16];
    let timestamp_ms_be = timestamp_ms.to_be_bytes();
    raw_uuid[..6].copy_from_slice(&timestamp_ms_be[2..]);
    raw_uuid[6..].copy_from_slice(&rand_data);

    raw_uuid[6] = (raw_uuid[6] & 0x0F) | 0x70;
    raw_uuid[8] = (raw_uuid[8] & 0x3F) | 0x80;

    Uuid::from_bytes(raw_uuid)
}

/// Process an ASCII-encoded Mastodon snowflake into a UUID v7 identifier
///
/// # Errors
///
/// - Parsing the Mastodon snowflake into a u64 failed
#[inline]
pub fn process<T>(masto_id: T) -> Result<Uuid, Error>
where
    T: AsRef<[u8]>,
{
    lexical::parse(masto_id)
        .map(process_u64)
        .map_err(Error::from)
}
