use async_std::prelude::*;
use async_std::task::{self, Context, Poll};

use std::pin::Pin;

use crate::ParallelStream;

pin_project_lite::pin_project! {
    /// Count the number of items of the stream.
    ///
    /// This `struct` is created by the [`count`] method on [`ParallelStream`]. See its
    /// documentation for more.
    ///
    /// [`count`]: trait.ParallelStream.html#method.count
    /// [`ParallelStream`]: trait.ParallelStream.html
    #[derive(Clone, Debug)]
    pub struct Count<S> {
        #[pin]
        stream: S,
        count: usize,
    }
}

impl<S: ParallelStream> Count<S> {
    pub(super) fn new(stream: S) -> Self {
        Self { stream, count: 0 }
    }
}

impl<S: ParallelStream> Future for Count<S> {
    type Output = usize;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match task::ready!(this.stream.poll_next(cx)) {
            None => Poll::Ready(*this.count),
            Some(_) => {
                *this.count += 1;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

#[async_std::test]
async fn smoke() {
    let s = async_std::stream::repeat(5usize);

    let cnt = crate::from_stream(s).take(10).count().await;

    assert_eq!(cnt, 10);
}
