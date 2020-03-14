//! Parallel async rust iterators
//!
//! # Examples
//!
//! ```
//! use parallel_stream::prelude::*;
//!
//! #[async_std::main]
//! async fn main() {
//!     let v = vec![1, 2, 3, 4];
//!     let mut stream = v.into_par_stream().map(|n| async move { n * n });
//!
//!     let mut out = vec![];
//!     while let Some(n) = stream.next().await {
//!         out.push(n);
//!     }
//!     out.sort();
//!
//!     assert_eq!(out, vec![1usize, 4, 9, 16]);
//! }
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

pub mod prelude;
pub mod vec;
