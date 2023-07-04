use rand::Rng;
use std::{fmt::Debug, future::Future, ops::RangeInclusive, time::Duration};

const ERROR_SLEEP_RANGE_SECS: RangeInclusive<f64> = 3.0..=6.0;

pub async fn sleep_a_bit() {
    let sleep_duration =
        Duration::from_secs_f64(rand::thread_rng().gen_range(ERROR_SLEEP_RANGE_SECS));

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
                tracing::error!(?error, "rerun iteration failed");
                sleep_a_bit().await;
            }
        }
    }
}
