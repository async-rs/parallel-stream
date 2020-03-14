use async_std::prelude::*;
use async_std::sync::{self, Receiver, Sender};
use async_std::task::{self, Context, Poll};

use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use crate::ParallelStream;

pin_project_lite::pin_project! {
    /// Call a closure on each element of the stream.
    #[derive(Debug)]
    pub struct ForEach {
        // Receiver that tracks whether all tasks have finished executing.
        #[pin]
        receiver: Receiver<()>,
        // Track whether the input stream has been exhausted.
        exhausted: Arc<AtomicBool>,
        // Count how many tasks are executing.
        ref_count: Arc<AtomicU64>,
        // Max concurrency limit.
        limit: Option<usize>,
    }
}

impl ForEach {
    /// Create a new instance of `ForEach`.
    pub fn new<S, F, Fut>(mut stream: S, mut f: F) -> Self
    where
        S: ParallelStream,
        F: FnMut(S::Item) -> Fut + Send + Sync + Copy + 'static,
        Fut: Future<Output = ()> + Send,
    {
        let exhausted = Arc::new(AtomicBool::new(false));
        let ref_count = Arc::new(AtomicU64::new(0));
        let (sender, receiver): (Sender<()>, Receiver<()>) = sync::channel(1);

        // Initialize the return type here to prevent borrowing issues.
        let this = Self {
            receiver,
            exhausted: exhausted.clone(),
            ref_count: ref_count.clone(),
            limit: stream.limit(),
        };

        task::spawn(async move {
            while let Some(item) = stream.next().await {
                let sender = sender.clone();
                let exhausted = exhausted.clone();
                let ref_count = ref_count.clone();

                ref_count.fetch_add(1, Ordering::SeqCst);

                task::spawn(async move {
                    // Execute the closure.
                    f(item).await;

                    // Wake up the receiver if we know we're done.
                    ref_count.fetch_sub(1, Ordering::SeqCst);
                    if exhausted.load(Ordering::SeqCst) && ref_count.load(Ordering::SeqCst) == 0 {
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

impl Future for ForEach {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        task::ready!(this.receiver.poll_next(cx));
        Poll::Ready(())
    }
}

#[async_std::test]
async fn smoke() {
    let s = async_std::stream::repeat(5usize);
    crate::from_stream(s)
        .take(3)
        .for_each(|n| async move {
            // TODO: assert that this is called 3 times.
            dbg!(n);
        })
        .await;
}
