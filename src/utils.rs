// use core::pin::Pin;
// use core::task::{Context, Poll};

// use std::sync::atomic::{AtomicUsize, Ordering};
// use std::sync::Arc;

// use async_std::stream::Stream;

// /// A stream that has a max concurrency of N.
// pub(crate) struct LimitStream {
//     limit: Option<usize>,
//     ref_count: Arc<AtomicUsize>,
// }

// impl LimitStream {
//     /// Create a new instance of LimitStream.
//     pub(crate) fn new(limit: Option<usize>) -> Self {
//         Self {
//             limit,
//             ref_count: Arc::new(AtomicUsize::new(0)),
//         }
//     }
// }

// #[derive(Debug)]
// pub(crate) struct Guard {
//     limit: Option<usize>,
//     ref_count: Arc<AtomicUsize>,
// }

// impl Guard {
//     fn new(limit: Option<usize>, ref_count: Arc<AtomicUsize>) -> Self {
//         Self { limit, ref_count }
//     }
// }

// impl Drop for Guard {
//     fn drop(&mut self) {
//         if self.limit.is_some() {
//             self.ref_count.fetch_sub(1, Ordering::SeqCst);
//         }
//     }
// }

// impl Stream for LimitStream {
//     type Item = Guard;

//     fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         if self.limit.is_none() {
//             let guard = Guard::new(self.limit, self.ref_count.clone());
//             return Poll::Ready(Some(guard));
//         }
//         todo!();
//     }
// }
