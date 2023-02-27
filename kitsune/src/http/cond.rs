use axum::handler::Handler;
use futures_util::{future::Either, FutureExt};
use http::{header::ACCEPT, Request};
use mime::TEXT_HTML;

/// Conditional wrapper around two handlers
///
/// If the conditional wrapper returns `true`, the left future is invoked.
/// Otherwise the right future is invoked.
#[derive(Clone)]
pub struct ConditionalWrapper<C, L, R> {
    condition: C,
    left: L,
    right: R,
}

impl<C, L, R> ConditionalWrapper<C, L, R> {
    pub fn new(condition: C, left: L, right: R) -> Self {
        Self {
            condition,
            left,
            right,
        }
    }
}

impl<C, L, R, T, S, B> Handler<T, S, B> for ConditionalWrapper<C, L, R>
where
    C: Fn(&Request<B>) -> bool + Clone + Send + 'static,
    L: Clone + Handler<T, S, B> + Send + 'static,
    R: Clone + Handler<T, S, B> + Send + 'static,
{
    type Future = Either<L::Future, R::Future>;

    fn call(self, req: Request<B>, state: S) -> Self::Future {
        if (self.condition)(&req) {
            self.left.call(req, state).left_future()
        } else {
            self.right.call(req, state).right_future()
        }
    }
}

pub fn html<B, L, R>(
    left: L,
    right: R,
) -> ConditionalWrapper<impl Fn(&Request<B>) -> bool + Clone + Send + 'static, L, R> {
    let cond = |req: &Request<B>| {
        req.headers()
            .get(ACCEPT)
            .and_then(|header| {
                header
                    .to_str()
                    .map(|value| value.contains(TEXT_HTML.as_ref()))
                    .ok()
            })
            .unwrap_or(false)
    };

    ConditionalWrapper::new(cond, left, right)
}
