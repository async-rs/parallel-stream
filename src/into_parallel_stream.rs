use crate::ParallelStream;

/// Conversion into a `ParallelStream`.
pub trait IntoParallelStream {
    /// The type of the elements being iterated over.
    type Item: Send;

    /// Which kind of stream are we turning this into?
    type IntoParStream: ParallelStream<Item = Self::Item>;

    /// Creates a parallel stream from a value.
    fn into_par_stream(self) -> Self::IntoParStream;
}

impl<I: ParallelStream> IntoParallelStream for I {
    type Item = I::Item;
    type IntoParStream = I;

    #[inline]
    fn into_par_stream(self) -> I {
        self
    }
}
