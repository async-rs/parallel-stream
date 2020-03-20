use async_std::future::Future;
use async_std::sync::{self, Receiver};
use async_std::task;
use async_std::task::{Context, Poll};
use core::pin::Pin;
use pin_project_lite::pin_project;

use crate::ParallelStream;

pin_project! {
    /// A parallel stream that yields elements by calling a closure.
    ///
    /// This stream is created by the [`from_fn`] function.
    /// See it documentation for more.
    ///
    /// [`from_fn`]: fn.from_fn.html
    ///
    /// # Examples
    #[derive(Clone, Debug)]
    pub struct FromFn<T, F> {
        #[pin]
        receiver: Receiver<T>,
        f: F,
        limit: Option<usize>,
    }
}

/// Creates a parallel stream from a closure.
pub fn from_fn<T, F, Fut>(mut f: F) -> FromFn<T, F>
where
    T: Send + Sync + Unpin + 'static,
    F: FnMut() -> Fut + Send + Sync + Copy + 'static,
    Fut: Future<Output = Option<T>> + Send,
{
    let (sender, receiver) = sync::channel(1);
    task::spawn(async move {
        let sender = sender.clone();
        while let Some(val) = f().await {
            sender.send(val).await;
        };
    });
    FromFn {
        f,
        receiver,
        limit: None,
    }
}

impl<T: Send + Sync + Unpin + 'static, F, Fut> ParallelStream for FromFn<T, F>
where
    T: Send + Sync + Unpin + 'static,
    F: FnMut() -> Fut + Send + Sync + Copy + 'static,
    Fut: Future<Output = Option<T>> + Send,
{
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

// #[async_std::test]
// async fn smoke() {
//     let mut output = vec![];
    
//     let mut stream = crate::from_fn(|| async move {
//        Some(1u8)
//     });
//     while let Some(n) = stream.next().await {
//         output.push(n);
//     }
//     assert_eq!(output, vec![1u8]);
// }
