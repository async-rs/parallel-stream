use core::pin::Pin;
use core::task::{Context, Poll};

use async_std::task::ready;
use pin_project_lite::pin_project;

use crate::ParallelStream;

pin_project! {
    /// A stream that skips the first `n` items of another stream.
    ///
    /// This `struct` is created by the [`skip`] method on [`ParallelStream`]. See its
    /// documentation for more.
    ///
    /// [`skip`]: trait.ParallelStream.html#method.take
    /// [`ParallelStream`]: trait.ParallelStream.html
    #[derive(Clone, Debug)]
    pub struct Skip<S> {
        #[pin]
        stream: S,
        skipped: usize,
        limit: Option<usize>,
    }
}

impl<S: ParallelStream> Skip<S> {
    pub(super) fn new(stream: S, skipped: usize) -> Self {
        Self {
            limit: stream.get_limit(),
            skipped,
            stream,
        }
    }
}

impl<S: ParallelStream> ParallelStream for Skip<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<S::Item>> {
        let this = self.project();
        if *this.skipped > 0 {
            let next = ready!(this.stream.poll_next(cx));
            match next {
                Some(_) => *this.skipped -= 1,
                None => *this.skipped = 0,
            }
            Poll::Ready(next)
        } else {
            Poll::Ready(None)
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
    let s = async_std::stream::from_iter(vec![1, 2, 3, 4, 5, 6]).skip(3);
    let mut output = vec![];
    let mut stream = crate::from_stream(s);
    while let Some(n) = stream.next().await {
        output.push(n);
    }
    assert_eq!(output, vec![4, 5, 6]);
}
