use core::pin::Pin;

use async_std::stream::{IntoStream, Stream};
use async_std::task::{Context, Poll};
use pin_project_lite::pin_project;

use crate::ParallelStream;

pin_project! {
    /// A parallel stream that was created from sequential stream.
    ///
    /// This stream is created by the [`from_stream`] function.
    /// See it documentation for more.
    ///
    /// [`from_stream`]: fn.from_stream.html
    #[derive(Clone, Debug)]
    pub struct FromStream<S> {
        #[pin]
        stream: S,
    }
}

/// Converts a stream into a parallel stream.
pub fn from_stream<S: IntoStream>(stream: S) -> FromStream<S::IntoStream>
where
    S: Send + Sync,
{
    FromStream {
        stream: stream.into_stream(),
    }
}

impl<S: Stream + Send + Sync + Unpin + 'static> ParallelStream for FromStream<S>
where
    S::Item: Send,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.stream.poll_next(cx)
    }
}
