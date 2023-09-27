use std::{any::Any, future::Future, pin::Pin, sync::OnceLock, thread};
use thiserror::Error;
use tokio::{
    sync::{mpsc, oneshot},
    task::LocalSet,
};

type LocalBoxFuture<'a, O> = Pin<Box<dyn Future<Output = O> + 'a>>;
type FutureProducer = Box<dyn FnOnce() -> LocalBoxFuture<'static, Box<dyn Any + Send>> + Send>;
type Task = (FutureProducer, oneshot::Sender<Box<dyn Any + Send>>);

const CHANNEL_CAPACITY: usize = 500;
static GLOBAL_SINGLE_THREADED_RUNTIME: OnceLock<mpsc::Sender<Task>> = OnceLock::new();

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Oneshot(#[from] oneshot::error::RecvError),

    #[error("Spawn error")]
    SpawnError,
}

pub async fn run<F, Fut>(func: F) -> Result<Box<Fut::Output>, Error>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future,
    <Fut as Future>::Output: Send + 'static,
{
    let runtime_handle = GLOBAL_SINGLE_THREADED_RUNTIME.get_or_init(|| {
        let (sender, mut receiver) = mpsc::channel::<Task>(CHANNEL_CAPACITY);

        thread::spawn(move || {
            let local_set = LocalSet::new();
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            local_set.block_on(&runtime, async move {
                while let Some((producer, sender)) = receiver.recv().await {
                    tokio::task::spawn_local(async move {
                        if sender.send(producer().await).is_err() {
                            tracing::debug!("failed to send value from single threaded executor");
                        }
                    });
                }
            });
        });

        sender
    });

    let (sender, receiver) = oneshot::channel();
    let closure = Box::new(|| {
        Box::pin(async move {
            let output = func().await;
            Box::new(output) as Box<dyn Any + Send>
        }) as _
    });

    runtime_handle
        .send((closure, sender))
        .await
        .map_err(|_| Error::SpawnError)?;

    let result = receiver.await?;

    Ok(result.downcast().unwrap())
}
