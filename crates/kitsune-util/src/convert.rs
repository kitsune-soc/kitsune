use iso8601_timestamp::Timestamp;
use speedy_uuid::{uuid, Uuid};

#[inline]
#[must_use]
pub fn timestamp_to_uuid(timestamp: Timestamp) -> Uuid {
    let seconds = timestamp
        .duration_since(Timestamp::UNIX_EPOCH)
        .whole_seconds();
    let uuid_timestamp =
        uuid::Timestamp::from_unix(uuid::NoContext, seconds as u64, timestamp.nanosecond());

    Uuid::new_v7(uuid_timestamp)
}
