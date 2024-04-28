use std::time::Duration;

pub const BLOCK_TIME: Duration = Duration::from_secs(2);
pub const MAX_RETRIES: u32 = 10;
pub const MIN_IDLE_TIME: Duration = Duration::from_secs(10 * 60);
