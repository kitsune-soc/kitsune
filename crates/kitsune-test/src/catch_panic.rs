use pin_project_lite::pin_project;
use std::{
    future::Future,
    ops::Not,
    panic::{self, AssertUnwindSafe},
    pin::Pin,
    task::{Context, Poll},
    thread,
};

pin_project! {
    pub struct CatchPanic<F> {
        #[pin]
        inner: F,
        polled_to_completion: bool,
    }
}

impl<F> CatchPanic<F> {
    #[allow(dead_code)]
    pub fn new(inner: F) -> Self {
        Self {
            inner,
            polled_to_completion: false,
        }
    }
}

impl<F> Future for CatchPanic<F>
where
    F: Future,
{
    type Output = thread::Result<F::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        assert!(
            this.polled_to_completion.not(),
            "Polled after driven to completion"
        );

        let poll_result =
            match panic::catch_unwind(AssertUnwindSafe(|| this.inner.as_mut().poll(cx))) {
                Ok(Poll::Ready(out)) => Poll::Ready(Ok(out)),
                Ok(Poll::Pending) => Poll::Pending,
                Err(err) => Poll::Ready(Err(err)),
            };
        *this.polled_to_completion = poll_result.is_ready();

        poll_result
    }
}
