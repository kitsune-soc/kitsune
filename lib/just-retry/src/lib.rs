use rand::Rng;
use std::{fmt::Debug, future::Future, ops::RangeInclusive, time::Duration};

const ERROR_SLEEP_RANGE_SECS: RangeInclusive<u64> = 3..=6;

pub async fn sleep_a_bit() {
    let sleep_duration = Duration::from_secs(rand::thread_rng().gen_range(ERROR_SLEEP_RANGE_SECS));
    tokio::time::sleep(sleep_duration).await;
}

pub async fn rerun_until_success<F, Fut, Ok, Err>(mut func: F) -> Ok
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<Ok, Err>>,
    Err: Debug,
{
    loop {
        match func().await {
            Ok(val) => break val,
            Err(error) => {
                tracing::error!(?error, "job completion routine failed");
                sleep_a_bit().await;
            }
        }
    }
}
