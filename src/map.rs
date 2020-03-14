use async_std::prelude::*;
use async_std::sync::{self, Receiver};
use async_std::task;

use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    /// A parallel stream that maps value of another stream with a function.
    #[derive(Debug)]
    pub struct Map<T> {
        #[pin]
        receiver: Receiver<T>,
    }
}

impl<T: Send + 'static> Stream for Map<T> {
    type Item = T;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.receiver.poll_next(cx)
    }
}

impl<T: Send + 'static> Map<T> {
    /// Create a new instance of `Map`.
    pub fn new<S, F, Fut>(mut input: S, mut f: F) -> Self
    where
        S: Stream + Send + Sync + Unpin + 'static,
        S::Item: Send,
        F: FnMut(S::Item) -> Fut + Send + Sync + Copy + 'static,
        Fut: Future<Output = T> + Send,
    {
        let (sender, receiver) = sync::channel(1);
        task::spawn(async move {
            while let Some(item) = input.next().await {
                let sender = sender.clone();
                task::spawn(async move {
                    let res = f(item).await;
                    sender.send(res).await;
                });
            }
        });
        Map { receiver }
    }
}

#[async_std::test]
async fn test_map() {
    let s = async_std::stream::repeat(5usize).take(3);
    let v: Vec<usize> = Map::new(s, |n| async move { n * 2 }).collect().await;
    assert_eq!(v, vec![10usize; 3]);
}
