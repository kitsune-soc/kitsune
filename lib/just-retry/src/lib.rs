#[macro_use]
extern crate tracing;

use retry_policies::{policies::ExponentialBackoff, Jitter, RetryDecision};
use std::{
    fmt::Debug,
    future::Future,
    ops::ControlFlow,
    time::{Duration, SystemTime},
};

pub use retry_policies;

/// Start time of the request
pub enum StartTime {
    /// Implies the start time is at `n`
    At(SystemTime),

    /// Implies the start time is irrelevant to the policy and we will imply pass  
    /// [`SystemTime::UNIX_EPOCH`] to it to avoid syscalls
    Irrelevant,
}

impl StartTime {
    fn as_time(&self) -> SystemTime {
        match self {
            Self::At(at) => *at,
            Self::Irrelevant => SystemTime::UNIX_EPOCH,
        }
    }
}

pub trait JustRetryPolicy: retry_policies::RetryPolicy {
    fn should_retry(
        &self,
        request_start_time: StartTime,
        n_past_retries: u32,
    ) -> ControlFlow<(), Duration>;
}

impl<T> JustRetryPolicy for T
where
    T: retry_policies::RetryPolicy,
{
    fn should_retry(
        &self,
        request_start_time: StartTime,
        n_past_retries: u32,
    ) -> ControlFlow<(), Duration> {
        if let RetryDecision::Retry { execute_after } =
            self.should_retry(request_start_time.as_time(), n_past_retries)
        {
            let delta = execute_after
                .duration_since(SystemTime::now())
                .expect("Some major clock fuckery happened");

            ControlFlow::Continue(delta)
        } else {
            ControlFlow::Break(())
        }
    }
}

pub trait RetryExt<T> {
    fn retry<R>(&mut self, retry_policy: R) -> impl Future<Output = T> + Send
    where
        R: JustRetryPolicy + Send;
}

impl<F, Fut, T, E> RetryExt<Fut::Output> for F
where
    F: FnMut() -> Fut + Send,
    Fut: Future<Output = Result<T, E>> + Send,
    T: Send,
    E: Debug + Send,
{
    #[instrument(skip_all)]
    async fn retry<R>(&mut self, retry_policy: R) -> Fut::Output
    where
        R: JustRetryPolicy + Send,
    {
        let start_time = SystemTime::now();
        let mut retry_count = 0;

        loop {
            let result = match (self)().await {
                val @ Ok(..) => break val,
                Err(error) => {
                    debug!(?error, retry_count, "run failed");
                    Err(error)
                }
            };

            if let ControlFlow::Continue(delta) =
                JustRetryPolicy::should_retry(&retry_policy, StartTime::At(start_time), retry_count)
            {
                debug!(?delta, "retrying after backoff");
                tokio::time::sleep(delta).await;
            } else {
                debug!("not retrying");
                break result;
            }

            retry_count += 1;
        }
    }
}

#[must_use]
pub fn backoff_policy() -> impl JustRetryPolicy {
    ExponentialBackoff::builder()
        .jitter(Jitter::Bounded)
        .build_with_total_retry_duration(Duration::from_secs(24 * 3600)) // Kill the retrying after 24 hours
}
