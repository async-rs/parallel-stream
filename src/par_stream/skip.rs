use async_std::sync::{self, Receiver};
use async_std::task;
use core::pin::Pin;
use core::task::{Context, Poll};
use pin_project_lite::pin_project;

use crate::ParallelStream;

pin_project! {
    /// A stream that skips the first `n` items of another stream.
    ///
    /// This `struct` is created by the [`skip`] method on [`ParallelStream`]. See its
    /// documentation for more.
    ///
    /// [`skip`]: trait.ParallelStream.html#method.skip
    /// [`ParallelStream`]: trait.ParallelStream.html
    #[derive(Clone, Debug)]
    pub struct Skip<T> {
        #[pin]
        receiver: Receiver<T>,
        limit: Option<usize>,
    }
}

impl<T: Send + 'static> Skip<T> {
    pub(super) fn new<S>(mut stream: S, mut skipped: usize) -> Self
    where
        S: ParallelStream
    {
        let limit = stream.get_limit();
        let (sender, receiver) = sync::channel(1);
        task::spawn(async move {
            while let Some(val) = stream.next().await {
                if skipped == 0 {
                    sender.send(val).await
                } else {
                    skipped -= 1;
                }
            }
        });

        Skip { limit, receiver }
    }
}

impl<T: Send + 'static> ParallelStream for Skip<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.receiver.poll_next(cx)
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
    let s = async_std::stream::from_iter(vec![1, 2, 3, 4, 5, 6]);
    let mut output = vec![];
    let mut stream = crate::from_stream(s).skip(3);
    while let Some(n) = stream.next().await {
        output.push(n);
    }
    assert_eq!(output, vec![4, 5, 6]);
}
