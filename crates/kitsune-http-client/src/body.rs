use bytes::Bytes;
use futures_util::{StreamExt, TryStream, TryStreamExt};
use http_body::Frame;
use http_body_util::StreamBody;
use std::{
    borrow::Cow,
    fmt::{self, Debug},
    pin::Pin,
    task::{self, Poll},
};
use tower::BoxError;

mod body_def {
    #![allow(missing_docs)]

    use super::{BoxError, Bytes, Frame, StreamBody};
    use futures_util::stream::BoxStream;
    use pin_project_lite::pin_project;

    pin_project! {
        #[project = BodyProj]
        // Body on a budget
        pub enum Body {
            // Empty body
            Empty,

            // Body consisting of a single chunk
            Full { data: Option<Bytes> },

            // Body backed by a `StreamBody`
            Stream {
                #[pin]
                stream: StreamBody<BoxStream<'static, Result<Frame<Bytes>, BoxError>>>
            },
        }
    }
}

pub use self::body_def::Body;
use self::body_def::BodyProj;

impl Default for Body {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
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
        Self::Full {
            data: Some(data.into()),
        }
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

        Self::Stream {
            stream: StreamBody::new(stream),
        }
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
            BodyProj::Full { data } => Poll::Ready(data.take().map(|data| Ok(Frame::data(data)))),
            BodyProj::Stream { stream } => stream.poll_frame(cx),
        }
    }
}
