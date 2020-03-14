//! Parallel async rust iterators
//!
//! # Examples
//!
//! ```
//! // tbi
//! ```

#![forbid(unsafe_code, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples)]

mod from_stream;
mod into_parallel_stream;
mod par_stream;

pub use from_stream::{from_stream, FromStream};
pub use into_parallel_stream::IntoParallelStream;
pub use par_stream::{ForEach, Map, NextFuture, ParallelStream, Take};

pub mod vec;
