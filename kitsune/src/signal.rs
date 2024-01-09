use tokio_util::sync::CancellationToken;

#[cfg(target_family = "unix")]
use tokio::signal::unix::SignalKind;

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(target_family = "unix")]
    let second_signal = async {
        let mut terminate = tokio::signal::unix::signal(SignalKind::terminate()).unwrap();
        let mut quit = tokio::signal::unix::signal(SignalKind::quit()).unwrap();

        tokio::select! {
            _ = terminate.recv() => (),
            _ = quit.recv() => (),
        }
    };
    #[cfg(not(target_family = "unix"))]
    let second_signal = std::future::pending();

    tokio::select! {
        _ = ctrl_c => (),
        () = second_signal => (),
    }
}

#[derive(Clone)]
pub struct Receiver {
    inner: CancellationToken,
}

impl Receiver {
    pub async fn wait(self) {
        self.inner.cancelled_owned().await;
    }
}

#[must_use]
pub fn shutdown() -> Receiver {
    let notifier = CancellationToken::new();

    {
        let notifier = notifier.clone();

        tokio::spawn(async move {
            shutdown_signal().await;
            notifier.cancel();
        });
    }

    Receiver { inner: notifier }
}
