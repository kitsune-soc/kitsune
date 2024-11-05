use bytes::Bytes;
use futures_util::{stream::BoxStream, StreamExt, TryStream, TryStreamExt};
use http_body::Frame;
use http_body_util::StreamBody;
use pin_project::pin_project;
use std::{
    borrow::Cow,
    fmt::{self, Debug},
    pin::Pin,
    task::{self, Poll},
};
use tower::BoxError;

/// Body on a budget
#[derive(Default)]
#[pin_project(project = BodyProj)]
pub enum Body {
    /// Empty body
    #[default]
    Empty,

    /// Body consisting of a single chunk
    Full(Option<Bytes>),

    /// Body backed by a `StreamBody`
    Stream(#[pin] StreamBody<BoxStream<'static, Result<Frame<Bytes>, BoxError>>>),
}

impl Body {
    /// Empty body
    #[inline]
    #[must_use]
    pub fn empty() -> Self {
        Self::Empty
    }

    /// Single byte chunk body
    #[inline]
    pub fn data<D>(data: D) -> Self
    where
        D: Into<Bytes>,
    {
        Self::Full(Some(data.into()))
    }

    /// Stream body
    #[inline]
    pub fn stream<S>(stream: S) -> Self
    where
        S: TryStream + Send + 'static,
        S::Ok: Into<Bytes>,
        S::Error: Into<BoxError>,
    {
        let stream = stream
            .map_ok(|chunk| Frame::data(chunk.into()))
            .map_err(Into::into)
            .boxed();

        Self::Stream(StreamBody::new(stream))
    }
}

impl Debug for Body {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .finish_non_exhaustive()
    }
}

impl From<Bytes> for Body {
    #[inline]
    fn from(value: Bytes) -> Self {
        Self::data(value)
    }
}

impl From<Cow<'_, str>> for Body {
    #[inline]
    fn from(value: Cow<'_, str>) -> Self {
        Self::data(value.into_owned())
    }
}

impl From<String> for Body {
    #[inline]
    fn from(value: String) -> Self {
        Self::data(value)
    }
}

impl From<Vec<u8>> for Body {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        Self::data(value)
    }
}

impl http_body::Body for Body {
    type Data = Bytes;
    type Error = BoxError;

    #[inline]
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match self.project() {
            BodyProj::Empty => Poll::Ready(None),
            BodyProj::Full(data) => Poll::Ready(data.take().map(|data| Ok(Frame::data(data)))),
            BodyProj::Stream(stream) => stream.poll_frame(cx),
        }
    }
}
