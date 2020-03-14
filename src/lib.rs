//! Parallel async rust iterators
//!
//! # Examples
//!
//! ```
//! // tbi
//! ```

#![forbid(unsafe_code, future_incompatible, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples, unreachable_pub)]

use async_std::prelude::*;
use async_std::sync;
use async_std::task;

use async_std::sync::{Receiver, Sender};

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use std::sync::Arc;

/// Parallel streaming map.
pub async fn par_map<S, F, T, Fut>(mut input: S, mut f: F) -> sync::Receiver<T>
where
    S: Stream + Send + Sync + 'static + Unpin,
    S::Item: Send,
    F: FnMut(S::Item) -> Fut + Send + Sync + Copy + 'static,
    T: Send + 'static,
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
    receiver
}

/// Executes `f` on each item in the stream, in parallel.
pub async fn par_for_each<S, F, Fut>(mut input: S, mut f: F)
where
    S: Stream + Send + Sync + 'static + Unpin,
    S::Item: Send,
    F: FnMut(S::Item) -> Fut + Send + Sync + Copy + 'static,
    Fut: Future<Output = ()> + Send,
{
    let exhausted = Arc::new(AtomicBool::new(false));
    let ref_count = Arc::new(AtomicU64::new(0));

    let (sender, receiver): (Sender<()>, Receiver<()>) = sync::channel(1);
    task::spawn(async move {
        while let Some(item) = input.next().await {
            let sender = sender.clone();
            let exhausted = exhausted.clone();
            let ref_count = ref_count.clone();
            ref_count.fetch_add(1, Ordering::SeqCst);
            task::spawn(async move {
                f(item).await;
                ref_count.fetch_sub(1, Ordering::SeqCst);
                if exhausted.load(Ordering::SeqCst) && ref_count.load(Ordering::SeqCst) == 0 {
                    sender.send(()).await;
                }
            });
        }
        exhausted.store(true, Ordering::SeqCst);
    });
    let _ = receiver.recv().await;
}

#[async_std::test]
async fn test_map() {
    let s = async_std::stream::repeat(5usize).take(3);
    let v: Vec<usize> = par_map(s, |n| async move { n * 2 }).await.collect().await;
    assert_eq!(v, vec![10usize, 10, 10]);
}

#[async_std::test]
async fn test_for_each() {
    let s = async_std::stream::repeat(5usize).take(3);
    par_for_each(s, |n| async move {
        dbg!(n);
    })
    .await;
}
