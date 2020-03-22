use core::pin::Pin;
use core::task::{Context, Poll};

use async_std::task::ready;
use pin_project_lite::pin_project;

use crate::ParallelStream;

pin_project! {
    /// A stream that yields the current count and element.
    ///
    /// This `struct` is created by the [`enumerate`] method on [`ParallelStream`]. See its
    /// documentation for more.
    ///
    /// [`enumerate`]: trait.ParallelStream.html#method.enumerate
    /// [`ParallelStream`]: trait.ParallelStream.html
    #[derive(Clone, Debug)]
    pub struct Enumerate<S> {
        #[pin]
        stream: S,
        count: usize,
        limit: Option<usize>,
    }
}

impl<S: ParallelStream> Enumerate<S> {
    pub(super) fn new(stream: S) -> Self {
        Self {
            limit: stream.get_limit(),
            count: 0,
            stream,
        }
    }
}

impl<S: ParallelStream> ParallelStream for Enumerate<S> {
    type Item = (usize, S::Item);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let next = ready!(this.stream.poll_next(cx));
        *this.count += 1;
        let count = *this.count;
        Poll::Ready(next.map(|val| (count, val)))
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
    let s = async_std::stream::repeat(5usize).enumerate().take(3);
    let mut output = vec![];
    let mut stream = crate::from_stream(s);
    while let Some(n) = stream.next().await {
        output.push(n);
    }
    assert_eq!(output, vec![(0, 5), (1, 5), (2, 5)]);
}
