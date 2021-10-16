// use async_std::prelude::*;
use async_std::future::Future;
use async_std::sync::{self, Receiver};
use async_std::task;

use std::pin::Pin;
use std::task::{Context, Poll};

use crate::ParallelStream;

pin_project_lite::pin_project! {
    /// A parallel stream that filters value of another stream with a function.
    #[derive(Debug)]
    pub struct Filter<S> where S: ParallelStream {
        #[pin]
        receiver: Receiver<S::Item>,
        limit: Option<usize>,
    }
}

impl<S> Filter<S>
where
    S: ParallelStream,
{
    /// Create a new instance of `Filter`.
    pub fn new<F, Fut>(mut stream: S, mut f: F) -> Self
    where
        S: ParallelStream,
        F: FnMut(&S::Item) -> Fut + Send + Sync + Copy + 'static,
        Fut: Future<Output = bool> + Send,
        S::Item: Sync,
    {
        let (sender, receiver) = sync::channel(1);
        let limit = stream.get_limit();
        task::spawn(async move {
            while let Some(item) = stream.next().await {
                let sender = sender.clone();
                task::spawn(async move {
                    let res = f(&item).await;
                    if res {
                        sender.send(item).await;
                    }
                });
            }
        });
        Filter { receiver, limit }
    }
}

impl<S> ParallelStream for Filter<S>
where
    S: ParallelStream,
{
    type Item = S::Item;
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
    let s = async_std::stream::from_iter(vec![2, 1, 2, 3, 2]);
    let mut output: Vec<usize> = vec![];
    let mut stream = crate::from_stream(s).filter(|&n| async move { n % 2 == 0 });
    while let Some(n) = stream.next().await {
        output.push(n);
    }
    assert_eq!(output, vec![2usize; 3]);
}
