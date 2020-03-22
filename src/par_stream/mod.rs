use async_std::future::Future;
use async_std::task::{Context, Poll};

use std::pin::Pin;

use crate::{FromParallelStream, IntoParallelStream};

pub use enumerate::Enumerate;
pub use for_each::ForEach;
pub use map::Map;
pub use next::NextFuture;
pub use take::Take;
pub use zip::Zip;

mod enumerate;
mod for_each;
mod map;
mod next;
mod take;
mod zip;

/// Parallel version of the standard `Stream` trait.
pub trait ParallelStream: Sized + Send + Sync + Unpin + 'static {
    /// The type of items yielded by this stream.
    type Item: Send;

    /// Attempts to receive the next item from the stream.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;

    /// Set a max concurrency limit
    fn limit(self, limit: impl Into<Option<usize>>) -> Self;

    /// Get the max concurrency limit
    fn get_limit(&self) -> Option<usize>;

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

    /// A stream that yields two streams simultaneously
    fn zip<S, T>(self, other: T) -> Zip<Self, <T as IntoParallelStream>::IntoParStream>
    where
        Self: Sized,
        T: IntoParallelStream,
    {
        Zip::new(self, other.into_par_stream())
    }

    /// A stream that yields a current count and element
    fn enumerate<S, T>(self) -> Enumerate<Self>
    where
        Self: Sized,
    {
        Enumerate::new(self)
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

    /// Transforms a stream into a collection.
    ///
    ///`collect()` can take anything streamable, and turn it into a relevant
    /// collection. This is one of the more powerful methods in the async
    /// standard library, used in a variety of contexts.
    fn collect<'a, B>(self) -> Pin<Box<dyn Future<Output = B> + 'a + Send>>
    where
        Self: Sized + 'a,
        B: FromParallelStream<Self::Item>,
    {
        FromParallelStream::from_par_stream(self)
    }
}
