use core::cmp;
use core::pin::Pin;
use core::task::{Context, Poll};

use async_std::task::ready;
use pin_project_lite::pin_project;

use crate::ParallelStream;

pin_project! {
    /// A stream that yields two streams simultaneously.
    ///
    /// This `struct` is created by the [`zip`] method on [`ParallelStream`]. See its
    /// documentation for more.
    ///
    /// [`zip`]: trait.ParallelStream.html#method.zip
    /// [`ParallelStream`]: trait.ParallelStream.html
    #[derive(Clone, Debug)]
    pub struct Zip<A, B> {
        #[pin]
        stream: A,
        #[pin]
        other: B,
        limit: Option<usize>,
    }
}

impl<A: ParallelStream, B: ParallelStream> Zip<A, B> {
    pub(super) fn new(stream: A, other: B) -> Self {
        Self {
            limit: cmp::min(stream.get_limit(), other.get_limit()),
            other,
            stream,
        }
    }
}

impl<A: ParallelStream, B: ParallelStream> ParallelStream for Zip<A, B> {
    type Item = (A::Item, B::Item);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let next = ready!(this.stream.poll_next(cx));
        let other_next = ready!(this.other.poll_next(cx));
        match (next, other_next) {
            (Some(a), Some(b)) => Poll::Ready(Some((a, b))),
            _ => Poll::Ready(None),
        }
    }

    fn limit(mut self, limit: impl Into<Option<usize>>) -> Self {
        self.limit = limit.into();
        self
    }

    fn get_limit(&self) -> Option<usize> {
        self.limit
    }
}

#[async_std::test]
async fn smoke() {
    use async_std::prelude::*;
    let s = async_std::stream::repeat(5usize)
        .zip(async_std::stream::repeat(10usize))
        .take(3);
    let mut output = vec![];
    let mut stream = crate::from_stream(s);
    while let Some(n) = stream.next().await {
        output.push(n);
    }
    assert_eq!(output, vec![(5, 10), (5, 10), (5, 10)]);
}
