//! Parallel async rust iterators
//!
//! # Examples
//!
//! ```
//! // tbi
//! ```

#![forbid(unsafe_code, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples, unreachable_pub)]

mod for_each;
mod map;
mod next;
mod parallel_stream;

pub use for_each::ForEach;
pub use map::Map;
pub use next::NextFuture;
pub use parallel_stream::ParallelStream;
