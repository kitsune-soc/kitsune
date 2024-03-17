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

#[cfg(test)]
mod test {
    #![allow(clippy::unreadable_literal)]

    use time::{Month, OffsetDateTime};

    // ID nabbed from this post: <https://hachyderm.io/@samhenrigold/112094325204679902>
    const ID: u64 = 112094325204679902;
    const ID_STR: &str = "112094325204679902";

    fn uuid_timestamp_to_time(timestamp: uuid::Timestamp) -> OffsetDateTime {
        let (seconds, nanos) = timestamp.to_unix();
        let nanos = ((seconds as i128) * 1_000_000_000) + (nanos as i128);
        OffsetDateTime::from_unix_timestamp_nanos(nanos).unwrap()
    }

    #[test]
    fn integer_convert_works() {
        let uuid = crate::process_u64(ID);
        let timestamp = uuid_timestamp_to_time(uuid.get_timestamp().unwrap());

        assert_eq!(timestamp.day(), 14);
        assert_eq!(timestamp.month(), Month::March);
        assert_eq!(timestamp.year(), 2024);

        assert_eq!(timestamp.hour(), 13);
        assert_eq!(timestamp.minute(), 41);
        assert_eq!(timestamp.second(), 3);
    }

    #[test]
    fn string_convert_works() {
        let uuid = crate::process(ID_STR).unwrap();
        let timestamp = uuid_timestamp_to_time(uuid.get_timestamp().unwrap());

        assert_eq!(timestamp.day(), 14);
        assert_eq!(timestamp.month(), Month::March);
        assert_eq!(timestamp.year(), 2024);

        assert_eq!(timestamp.hour(), 13);
        assert_eq!(timestamp.minute(), 41);
        assert_eq!(timestamp.second(), 3);
    }
}
