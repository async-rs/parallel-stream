use core::future::Future;
use core::pin::Pin;

use crate::IntoParallelStream;

/// Conversion from a `ParallelStream`.
pub trait FromParallelStream<T: Send> {
    /// Creates a value from a stream.
    fn from_par_stream<'a, S: IntoParallelStream<Item = T> + 'a>(
        stream: S,
    ) -> Pin<Box<dyn Future<Output = Self> + 'a>>;
}
