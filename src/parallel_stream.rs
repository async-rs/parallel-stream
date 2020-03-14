use async_std::future::Future;
use async_std::task::{Context, Poll};

use std::pin::Pin;

use crate::ForEach;
use crate::Map;
use crate::NextFuture;

/// Parallel version of the standard `Stream` trait.
pub trait ParallelStream: Sized + Send + Sync + Unpin + 'static {
    /// The type of items yielded by this stream.
    type Item: Send;

    /// Attempts to receive the next item from the stream.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;

    /// Applies `f` to each item of this stream in parallel, producing a new
    /// stream with the results.
    fn map<F, T, Fut>(self, f: F) -> Map<T>
    where
        F: FnMut(Self::Item) -> Fut + Send + Sync + Copy + 'static,
        T: Send + 'static,
        Fut: Future<Output = T> + Send,
    {
        Map::new(self, f)
    }

    /// Applies `f` to each item of this stream in parallel, producing a new
    /// stream with the results.
    fn next(&mut self) -> NextFuture<'_, Self> {
        NextFuture::new(self)
    }

    /// Applies `f` to each item of this stream in parallel.
    fn for_each<F, T, Fut>(self, f: F) -> ForEach
    where
        F: FnMut(Self::Item) -> Fut + Send + Sync + Copy + 'static,
        Fut: Future<Output = ()> + Send,
    {
        ForEach::new(self, f)
    }
}
