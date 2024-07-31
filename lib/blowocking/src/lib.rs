#[macro_use]
extern crate tracing;

use rayon::ThreadPool;
use thiserror::Error;
use tokio::sync::oneshot;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Oneshot(#[from] oneshot::error::RecvError),

    #[error(transparent)]
    #[cfg(feature = "io")]
    TokioJoin(#[from] tokio::task::JoinError),
}

#[inline]
async fn run_blocking<F, O>(pool: &ThreadPool, func: F) -> Result<O, Error>
where
    F: FnOnce() -> O + Send + 'static,
    O: Send + 'static,
{
    let (sender, receiver) = oneshot::channel();

    pool.spawn(move || {
        let _span = info_span!("rayon-worker", id = %rayon::current_thread_index().unwrap());

        let out = func();

        if sender.send(out).is_err() {
            debug!("Failed to send back value from rayon threadpool");
        }
    });

    receiver.await.map_err(Error::from)
}

macro_rules! define_rayon_pool {
    (name: $name:ident, description: $description:literal) => {
        #[inline]
        #[doc = $description]
        pub async fn $name<F, O>(func: F) -> Result<O, Error>
        where
            F: FnOnce() -> O + Send + 'static,
            O: Send + 'static,
        {
            use std::sync::LazyLock;

            static POOL: LazyLock<rayon::ThreadPool> = LazyLock::new(|| {
                rayon::ThreadPoolBuilder::new()
                    .build()
                    .expect("Failed to build rayon threadpool")
            });

            $crate::run_blocking(&POOL, func).await
        }
    };
}

define_rayon_pool! {
    name: cpu,
    description: "Spawn general-purpose CPU bound work (image conversion, compression, etc.)"
}

define_rayon_pool! {
    name: crypto,
    description: "Spawn cryptography-related work (signature creation/verification, password hashing, etc)"
}

/// Spawn I/O-bound blocking work (blocking filesystem operations, blocking network operations, etc.)
#[inline]
#[cfg(feature = "io")]
pub async fn io<F, O>(func: F) -> Result<O, Error>
where
    F: FnOnce() -> O + Send + 'static,
    O: Send + 'static,
{
    tokio::task::spawn_blocking(func).await.map_err(Error::from)
}
