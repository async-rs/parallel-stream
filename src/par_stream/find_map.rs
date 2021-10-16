// use async_std::prelude::*;
use async_std::future::Future;
use async_std::sync::{self, Receiver};
use async_std::task;

use std::pin::Pin;
use std::task::{Context, Poll};

use crate::ParallelStream;

pin_project_lite::pin_project! {
    /// A parallel stream that FindMaps value of another stream with a function.
    #[derive(Debug)]
    pub struct FindMap<T> {
        #[pin]
        receiver: Receiver<T>,
        limit: Option<usize>,
        already_sent: bool,
    }
}

impl<T> FindMap<T>
where
    T: Send + 'static,
{
    /// Create a new instance of `FindMap`.
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
                    let res = f(item).await;
                    if let Some(res) = res {
                        sender.send(res).await;
                    }
                });
            }
        });
        FindMap {
            receiver,
            limit,
            already_sent: false,
        }
    }
}

impl<T> ParallelStream for FindMap<T>
where
    T: Send + 'static,
{
    type Item = T;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use async_std::prelude::*;
        let this = self.project();
        if *this.already_sent {
            return Poll::Ready(None);
        }
        if let Poll::Ready(elt) = this.receiver.poll_next(cx) {
            if let Some(elt) = elt {
                *this.already_sent = true;
                return Poll::Ready(Some(elt));
            }
            return Poll::Ready(None);
        }
        Poll::Pending
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
    let s = async_std::stream::from_iter(vec![1, 1, 2, 3, 2]);
    let mut output: Vec<usize> = vec![];
    let mut stream = crate::from_stream(s).find_map(|n| async move {
        if n % 2 == 0 {
            Some(42usize)
        } else {
            None
        }
    });
    while let Some(n) = stream.next().await {
        output.push(n);
    }
    assert_eq!(output, vec![42]);
}
