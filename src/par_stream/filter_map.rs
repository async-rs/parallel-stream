// use async_std::prelude::*;
use async_std::future::Future;
use async_std::sync::{self, Receiver};
use async_std::task;

use std::pin::Pin;
use std::task::{Context, Poll};

use crate::ParallelStream;

pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct FilterMap<T> {
        #[pin]
        receiver: Receiver<T>,
        limit: Option<usize>,
    }
}

impl<T: Send + 'static> FilterMap<T> {
    /// Create a new instance of `FilterMap`.
    pub fn new<S, F, Fut>(mut stream: S, mut f: F) -> Self
    where
        S: ParallelStream,
        F: FnMut(S::Item) -> Fut + Send + Sync + Copy + 'static,
        Fut: Future<Output = Option<T>> + Send,
    {
        let (sender, receiver) = sync::channel(1);
        let limit = stream.get_limit();
        task::spawn(async move {
            while let Some(item) = stream.next().await {
                let sender = sender.clone();
                task::spawn(async move {
                    if let Some(res) = f(item).await {
                        sender.send(res).await;
                    }
                });
            }
        });
        FilterMap { receiver, limit }
    }
}

impl<T: Send + 'static> ParallelStream for FilterMap<T> {
    type Item = T;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use async_std::prelude::*;
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
    let s = async_std::stream::from_iter(vec![1, 2, 1, 2, 1, 2]);
    let mut output: Vec<usize> = vec![];
    let mut stream = crate::from_stream(s).filter_map(|n| async move {
        if n % 2 == 0 {
            Some(n)
        } else {
            None
        }
    });
    while let Some(n) = stream.next().await {
        output.push(n);
    }
    assert_eq!(output, vec![2usize; 3]);
}
