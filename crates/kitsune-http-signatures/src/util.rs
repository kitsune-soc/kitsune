use std::time::{Duration, SystemTime, SystemTimeError};

pub trait UnixTimestampExt {
    type Error;

    fn from_unix_timestamp(timestamp: u64) -> Self;
    fn to_unix_timestamp(&self) -> Result<u64, Self::Error>;
}

impl UnixTimestampExt for SystemTime {
    type Error = SystemTimeError;

    fn from_unix_timestamp(timestamp: u64) -> Self {
        let duration = Duration::from_secs(timestamp);
        SystemTime::UNIX_EPOCH + duration
    }

    fn to_unix_timestamp(&self) -> Result<u64, Self::Error> {
        self.duration_since(SystemTime::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
    }
}
