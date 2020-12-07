use async_std::future::Future;
use async_std::prelude::*;
use async_std::sync::{self, Receiver, Sender};
use async_std::task::{self, Context, Poll};
use pin_project_lite::pin_project;

use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use crate::ParallelStream;

pin_project! {
    /// Calls a closure on each element until true or exhausted.
    #[derive(Debug)]
    pub struct Any {
        #[pin]
        receiver: Receiver<()>,
        // Track whether the input stream has been exhausted.
        exhausted: Arc<AtomicBool>,
        // Count how many tasks are executing.
        ref_count: Arc<AtomicU64>,
        // Track the boolean value as executed.
        value: Arc<AtomicBool>,
    }
}

impl Any {
    /// Creates a new instance of `Any`.
    pub fn new<S, F, Fut>(mut stream: S, mut f: F) -> Self
    where
        S: ParallelStream,
        F: FnMut(S::Item) -> Fut + Send + Sync + Copy + 'static,
        Fut: Future<Output = bool> + Send,
    {
        let exhausted = Arc::new(AtomicBool::new(false));
        let value = Arc::new(AtomicBool::new(false));
        let ref_count = Arc::new(AtomicU64::new(0));
        let (sender, receiver): (Sender<()>, Receiver<()>) = sync::channel(1);
        let _limit = stream.get_limit();

        // Initialize the return type here to prevent borrowing issues.
        let this = Self {
            receiver,
            exhausted: exhausted.clone(),
            ref_count: ref_count.clone(),
            value: value.clone(),
        };

        task::spawn(async move {
            while let Some(item) = stream.next().await {
                let sender = sender.clone();
                let exhausted = exhausted.clone();
                let ref_count = ref_count.clone();
                let value = value.clone();

                ref_count.fetch_add(1, Ordering::SeqCst);

                task::spawn(async move {
                    // Execute the closure.
                    let res = f(item).await;
                    value.fetch_or(res, Ordering::SeqCst);

                    // Wake up the receiver if we know we're done.
                    ref_count.fetch_sub(1, Ordering::SeqCst);
                    if value.load(Ordering::SeqCst)
                        || (exhausted.load(Ordering::SeqCst)
                            && ref_count.load(Ordering::SeqCst) == 0)
                    {
                        sender.send(()).await;
                    }
                });
            }

            // The input stream will no longer yield items.
            exhausted.store(true, Ordering::SeqCst);
        });

        this
    }
}

impl Future for Any {
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        task::ready!(this.receiver.poll_next(cx));
        Poll::Ready(this.value.load(Ordering::SeqCst))
    }
}

#[async_std::test]
async fn smoke() {
    let s = async_std::stream::from_iter(vec![6, 9, 0, 7, 10]);
    let result = crate::from_stream(s)
        .any(|n| async move { n * 2 < 9 })
        .await;

    assert!(result);
}
