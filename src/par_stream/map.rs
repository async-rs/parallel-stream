// use async_std::prelude::*;
use async_std::future::Future;
use async_std::sync::{self, Receiver};
use async_std::task;

use std::pin::Pin;
use std::task::{Context, Poll};

use crate::ParallelStream;

pin_project_lite::pin_project! {
    /// A parallel stream that maps value of another stream with a function.
    #[derive(Debug)]
    pub struct Map<T> {
        #[pin]
        receiver: Receiver<T>,
        limit: Option<usize>,
    }
}

impl<T: Send + 'static> Map<T> {
    /// Create a new instance of `Map`.
    pub fn new<S, F, Fut>(mut stream: S, mut f: F) -> Self
    where
        S: ParallelStream,
        F: FnMut(S::Item) -> Fut + Send + Sync + Copy + 'static,
        Fut: Future<Output = T> + Send,
    {
        let (sender, receiver) = sync::channel(1);
        let limit = stream.limit();
        task::spawn(async move {
            while let Some(item) = stream.next().await {
                let sender = sender.clone();
                task::spawn(async move {
                    let res = f(item).await;
                    sender.send(res).await;
                });
            }
        });
        Map { receiver, limit }
    }
}

impl<T: Send + 'static> ParallelStream for Map<T> {
    type Item = T;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use async_std::prelude::*;
        let this = self.project();
        this.receiver.poll_next(cx)
    }

    fn set_limit(mut self, limit: impl Into<Option<usize>>) -> Self {
        self.limit = limit.into();
        self
    }

    fn limit(&self) -> Option<usize> {
        self.limit
    }
}

#[async_std::test]
async fn smoke() {
    use async_std::prelude::*;
    let s = async_std::stream::repeat(5usize).take(3);
    let mut output = vec![];
    let mut stream = crate::from_stream(s).map(|n| async move { n * 2 });
    while let Some(n) = stream.next().await {
        output.push(n);
    }
    assert_eq!(output, vec![10usize; 3]);
}
