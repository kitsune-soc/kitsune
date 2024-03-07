#![doc = include_str!("../README.md")]
#![forbid(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt;
use nanorand::{Rng, WyRand};
use uuid::Uuid;

/// Combined error type
#[derive(Debug)]
pub enum Error {
    /// Number parsing error
    NumberParse(atoi_radix10::ParseIntErrorPublic),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<atoi_radix10::ParseIntErrorPublic> for Error {
    fn from(value: atoi_radix10::ParseIntErrorPublic) -> Self {
        Self::NumberParse(value)
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

    uuid::Builder::from_unix_timestamp_millis(timestamp_ms, &rand_data).into_uuid()
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
    let result = atoi_radix10::parse(masto_id.as_ref())?;
    Ok(process_u64(result))
}
