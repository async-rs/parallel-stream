//! Parallel types for `Vec`.
//!
//! You will rarely need to interact with this module directly unless you need to
//! name one of the stream types.

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::{from_stream, FromParallelStream, FromStream, IntoParallelStream, ParallelStream};

use async_std::stream::{from_iter, FromIter};
use std::vec;

pin_project_lite::pin_project! {
    /// Parallel stream that moves out of a vector.
    #[derive(Debug)]
    pub struct IntoParStream<T> {
        #[pin]
        stream: FromStream<FromIter<vec::IntoIter<T>>>,
        limit: Option<usize>,
    }
}

impl<T: Send + Sync + 'static> ParallelStream for IntoParStream<T> {
    type Item = T;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.stream.poll_next(cx)
    }

    fn limit(mut self, limit: impl Into<Option<usize>>) -> Self {
        self.limit = limit.into();
        self
    }

    fn get_limit(&self) -> Option<usize> {
        self.limit
    }
}

impl<T: Send + Sync + 'static> IntoParallelStream for Vec<T> {
    type Item = T;
    type IntoParStream = IntoParStream<T>;

    #[inline]
    fn into_par_stream(self) -> Self::IntoParStream {
        IntoParStream {
            stream: from_stream(from_iter(self)),
            limit: None,
        }
    }
}

/// Collect items from a parallel stream into a vector.
///
/// # Examples
/// ```
/// use parallel_stream::prelude::*;
///
/// #[async_std::main]
/// async fn main() {
///     let v = vec![1, 2, 3, 4];
///     let mut stream = v.into_par_stream().map(|n| async move { n * n });
///     let mut res = Vec::from_par_stream(stream).await;
///     res.sort();
///     assert_eq!(res, vec![1, 4, 9, 16]);
/// }
///
impl<T: Send> FromParallelStream<T> for Vec<T> {
    fn from_par_stream<'a, S>(stream: S) -> Pin<Box<dyn Future<Output = Self> + 'a>>
    where
        S: IntoParallelStream<Item = T> + 'a,
    {
        Box::pin(async move {
            let mut stream = stream.into_par_stream();
            let mut res = Vec::with_capacity(0);
            while let Some(item) = stream.next().await {
                res.push(item);
            }
            res
        })
    }
}

#[async_std::test]
async fn smoke() {
    use crate::IntoParallelStream;

    let v = vec![1, 2, 3, 4];
    let mut stream = v.into_par_stream().map(|n| async move { n * n });

    let mut out = vec![];
    while let Some(n) = stream.next().await {
        out.push(n);
    }
    out.sort();

    assert_eq!(out, vec![1usize, 4, 9, 16]);
}
