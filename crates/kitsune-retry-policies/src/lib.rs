use futures_retry_policies::{retry_policies::RetryPolicies, tracing::Traced};
use retry_policies::{policies::ExponentialBackoff, Jitter};
use std::{
    fmt::{self, Debug},
    ops::ControlFlow,
    time::{Duration, SystemTime},
};

pub use futures_retry_policies::{tokio::RetryFutureExt, RetryPolicy};

pub struct NeverRetry<T>(T);

impl<T> Debug for NeverRetry<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> futures_retry_policies::ShouldRetry for NeverRetry<T> {
    fn should_retry(&self, _attempts: u32) -> bool {
        false
    }
}

impl<Res, T> futures_retry_policies::RetryPolicy<Res> for NeverRetry<T>
where
    T: futures_retry_policies::RetryPolicy<NeverRetry<Res>>,
{
    fn should_retry(&mut self, result: Res) -> ControlFlow<Res, Duration> {
        match self.0.should_retry(NeverRetry(result)) {
            ControlFlow::Break(NeverRetry(val)) => ControlFlow::Break(val),
            ControlFlow::Continue(dur) => ControlFlow::Continue(dur),
        }
    }
}

#[must_use]
pub fn futures_backoff_policy<Res>() -> impl futures_retry_policies::RetryPolicy<Res>
where
    Res: Debug,
{
    Traced(NeverRetry(RetryPolicies::new(backoff_policy())))
}

#[must_use]
pub fn backoff_policy() -> impl retry_policies::RetryPolicy {
    ExponentialBackoff::builder()
        .jitter(Jitter::Bounded)
        .build_with_total_retry_duration(Duration::from_secs(24 * 3600)) // Kill the retrying after 24 hours
        .for_task_started_at(SystemTime::now().into())
}
