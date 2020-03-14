use core::future::Future;
use core::pin::Pin;

use crate::IntoParallelStream;

/// Conversion from a `ParallelStream`.
pub trait FromParallelStream<T: Send> {
    /// Creates a value from a stream.
    fn from_par_stream<'a, S: IntoParallelStream<Item = T> + 'a>(
        stream: S,
    ) -> Pin<Box<dyn Future<Output = Self> + 'a + Send>>
    where
        S: Send;
}

#[async_std::test]
async fn is_send() {
    use crate::prelude::*;
    async_std::task::spawn(async move {
        let v: Vec<usize> = vec![1, 2, 3, 4];
        let stream = v.into_par_stream().map(|n| async move { n * n });
        let mut res = Vec::from_par_stream(stream).await;
        res.sort();
        assert_eq!(res, vec![1, 4, 9, 16]);
    })
    .await;
}
