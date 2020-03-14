use async_std::future::Future;
use async_std::task::{Context, Poll};

use std::pin::Pin;

pub use for_each::ForEach;
pub use map::Map;
pub use next::NextFuture;
pub use take::Take;

mod for_each;
mod map;
mod next;
mod take;

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

    /// Creates a stream that yields its first `n` elements.
    fn take(self, n: usize) -> Take<Self>
    where
        Self: Sized,
    {
        Take::new(self, n)
    }

    /// Applies `f` to each item of this stream in parallel.
    fn for_each<F, Fut>(self, f: F) -> ForEach
    where
        F: FnMut(Self::Item) -> Fut + Send + Sync + Copy + 'static,
        Fut: Future<Output = ()> + Send,
    {
        ForEach::new(self, f)
    }
}
