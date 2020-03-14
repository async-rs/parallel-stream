use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::ParallelStream;

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct NextFuture<'a, S: Unpin + ?Sized> {
    stream: &'a mut S,
}

impl<'a, S: ParallelStream + Unpin + ?Sized> NextFuture<'a, S> {
    pub(crate) fn new(stream: &'a mut S) -> Self {
        Self { stream }
    }
}

impl<S: ParallelStream + Unpin + ?Sized> Future for NextFuture<'_, S> {
    type Output = Option<S::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.stream).poll_next(cx)
    }
}
