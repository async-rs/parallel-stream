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
    pub struct Zip<A, B>
    where
        A: ParallelStream,
        B: ParallelStream,
    {
        #[pin]
        stream: A,
        #[pin]
        other: B,
        limit: Option<usize>,
        queued1: Option<A::Item>,
        queued2: Option<B::Item>,
    }
}

impl<A, B> Zip<A, B>
where
    A: ParallelStream,
    B: ParallelStream,
{
    pub(super) fn new(stream: A, other: B) -> Self {
        Self {
            limit: cmp::min(stream.get_limit(), other.get_limit()),
            other,
            stream,
            queued1: None,
            queued2: None,
        }
    }
}

impl<A, B> ParallelStream for Zip<A, B>
where
    A: ParallelStream,
    B: ParallelStream,
    A::Item: Sync,
    B::Item: Sync,
{
    type Item = (A::Item, B::Item);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut stream_done = false;
        let mut other_done = false;
        if this.queued1.is_none() {
            match this.stream.poll_next(cx) {
                Poll::Ready(Some(item1)) => *this.queued1 = Some(item1),
                Poll::Ready(None) | Poll::Pending => {
                    stream_done = true;
                }
            }
        }
        if this.queued2.is_none() {
            match this.other.poll_next(cx) {
                Poll::Ready(Some(item2)) => *this.queued2 = Some(item2),
                Poll::Ready(None) | Poll::Pending => {
                    other_done = true;
                }
            }
        }

        if this.queued1.is_some() && this.queued2.is_some() {
            let pair = (this.queued1.take().unwrap(), this.queued2.take().unwrap());
            Poll::Ready(Some(pair))
        } else if stream_done && other_done {
            Poll::Ready(None)
        } else {
            Poll::Pending
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
