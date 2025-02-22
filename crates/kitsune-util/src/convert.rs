use iso8601_timestamp::Timestamp;
use speedy_uuid::{Uuid, uuid};

#[inline]
#[must_use]
pub fn timestamp_to_uuid(timestamp: Timestamp) -> Uuid {
    let seconds = timestamp
        .duration_since(Timestamp::UNIX_EPOCH)
        .whole_seconds();

    Uuid::new_v7(uuid::Timestamp::from_unix(
        uuid::NoContext,
        seconds as u64,
        timestamp.nanosecond(),
    ))
}
