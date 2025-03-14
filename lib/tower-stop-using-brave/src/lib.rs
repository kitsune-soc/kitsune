use either::Either;
use http::{
    HeaderValue, Request, Response, StatusCode,
    header::{LOCATION, USER_AGENT},
};
use regex::Regex;
use std::{
    future::{self, Ready},
    sync::LazyLock,
    task::{self, Poll},
};
use tower::{Layer, Service};

static REDIRECT_URL: &str = "https://www.spacebar.news/stop-using-brave-browser/";
static USER_AGENT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(^|\s)Brave(/|\s)").expect("[Bug] Failed to compile User-Agent regex")
});

#[derive(Clone)]
pub struct StopUsingBraveService<S> {
    inner: S,
}

impl<S> StopUsingBraveService<S> {
    #[inline]
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for StopUsingBraveService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Default,
{
    type Error = S::Error;
    type Response = S::Response;
    type Future = Either<S::Future, Ready<Result<S::Response, S::Error>>>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        if let Some(Ok(user_agent)) = req.headers().get(USER_AGENT).map(HeaderValue::to_str) {
            if USER_AGENT_REGEX.is_match(user_agent) {
                let response = Response::builder()
                    .status(StatusCode::FOUND)
                    .header(LOCATION, REDIRECT_URL)
                    .body(ResBody::default())
                    .unwrap();

                return Either::Right(future::ready(Ok(response)));
            }
        }

        Either::Left(self.inner.call(req))
    }
}

#[derive(Clone, Default)]
pub struct StopUsingBraveLayer {
    _priv: (),
}

impl<S> Layer<S> for StopUsingBraveLayer {
    type Service = StopUsingBraveService<S>;

    #[inline]
    fn layer(&self, inner: S) -> Self::Service {
        StopUsingBraveService::new(inner)
    }
}

#[cfg(test)]
mod test {
    use crate::{REDIRECT_URL, StopUsingBraveLayer};
    use http::{
        Request, Response, StatusCode,
        header::{LOCATION, USER_AGENT},
    };
    use std::convert::Infallible;
    use tower::{Layer, ServiceExt, service_fn};

    const BRAVE_USER_AGENTS: &[&str] = &[
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Brave/120.0.0.0",
        "Mozilla/5.0 (Android 13.0.0; ) AppleWebKit/537.36 (KHTML, like Gecko) Brave/120 Chrome/120 Not_A Brand/8 Mobile Safari/537.36",
        "Mozilla/5.0 (Linux; Android 14; SM-S918U1) AppleWebKit/606.2.15 (KHTML, like Gecko) Brave/119.0.6045.134 Mobile Safari/606.2.15",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_14_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/75.0.3770.38 Safari/537.36 Brave/75",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/75.0.3770.38 Safari/537.36 Brave/75",
        "Mozilla/5.0 (iPad; CPU OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Brave/1.33.81 Mobile Safari/605.1.15",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 9_3_4 like Mac OS X) AppleWebKit/601.1.46 (KHTML, like Gecko) Brave/1.2.11 Mobile/13G35 Safari/601.1.46 _id/000002",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/537.36 (KHTML, like Gecko) Brave Chrome/86.0.4240.198 Safari/537.36",
    ];

    const OTHER_USER_AGENTS: &[&str] = &[
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/121.0",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 14.2; rv:109.0) Gecko/20100101 Firefox/121.0",
        "Mozilla/5.0 (X11; Linux i686; rv:109.0) Gecko/20100101 Firefox/121.0",
        "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/121.0",
        "Mozilla/5.0 (X11; Fedora; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/121.0",
    ];

    #[futures_test::test]
    async fn matches_brave_agents() {
        for user_agent in BRAVE_USER_AGENTS {
            let service = StopUsingBraveLayer::default().layer(service_fn(|_req: Request<()>| {
                // The "unreachable" expression provides type annotations for the compiler to figure out the response and error types
                #[allow(unreachable_code)]
                async move {
                    panic!("Shouldn't have reached the handler!")
                        as Result<Response<()>, Infallible>
                }
            }));

            let response = {
                let request = Request::builder()
                    .header(USER_AGENT, *user_agent)
                    .body(())
                    .unwrap();

                service.oneshot(request).await.unwrap()
            };

            assert_eq!(response.status(), StatusCode::FOUND);
            assert_eq!(
                response.headers().get(LOCATION).unwrap().as_bytes(),
                REDIRECT_URL.as_bytes()
            );
        }
    }

    #[futures_test::test]
    async fn doesnt_match_other_agents() {
        for user_agent in OTHER_USER_AGENTS {
            let service =
                StopUsingBraveLayer::default().layer(service_fn(|_req: Request<()>| async move {
                    Ok::<_, Infallible>(
                        Response::builder().status(StatusCode::OK).body(()).unwrap(),
                    )
                }));

            let response = {
                let request = Request::builder()
                    .header(USER_AGENT, *user_agent)
                    .body(())
                    .unwrap();

                service.oneshot(request).await.unwrap()
            };

            assert_eq!(response.status(), StatusCode::OK);
        }
    }
}
