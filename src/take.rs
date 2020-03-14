use core::pin::Pin;
use core::task::{Context, Poll};

use async_std::task::ready;
use pin_project_lite::pin_project;

use crate::ParallelStream;

pin_project! {
    /// A stream that yields the first `n` items of another stream.
    ///
    /// This `struct` is created by the [`take`] method on [`ParallelStream`]. See its
    /// documentation for more.
    ///
    /// [`take`]: trait.ParallelStream.html#method.take
    /// [`ParallelStream`]: trait.ParallelStream.html
    #[derive(Clone, Debug)]
    pub struct Take<S> {
        #[pin]
        pub(crate) stream: S,
        pub(crate) remaining: usize,
    }
}

impl<S> Take<S> {
    pub(super) fn new(stream: S, remaining: usize) -> Self {
        Self { stream, remaining }
    }
}

impl<S: ParallelStream> ParallelStream for Take<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<S::Item>> {
        let this = self.project();
        if *this.remaining == 0 {
            Poll::Ready(None)
        } else {
            let next = ready!(this.stream.poll_next(cx));
            match next {
                Some(_) => *this.remaining -= 1,
                None => *this.remaining = 0,
            }
            Poll::Ready(next)
        }
    }
}
