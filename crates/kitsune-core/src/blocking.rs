use crate::error::{Error, Result};
use tokio::sync::oneshot;

#[allow(clippy::missing_panics_doc)]
pub async fn cpu<F, O>(func: F) -> Result<O>
where
    F: FnOnce() -> O + Send + 'static,
    O: Send + 'static,
{
    let (sender, receiver) = oneshot::channel();

    rayon::spawn(move || {
        let _span = info_span!("rayon-worker", id = %rayon::current_thread_index().unwrap());

        let out = func();

        if sender.send(out).is_err() {
            error!("Failed to send back value from rayon threadpool");
        }
    });

    receiver.await.map_err(Error::from)
}
