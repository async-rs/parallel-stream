//! Data parallelism library for async-std.
//!
//! This library provides convenient parallel iteration of
//! [`Streams`](https://docs.rs/futures-core). Analogous to how
//! [Rayon](https://docs.rs/rayon/) provides parallel iteration of
//! `Iterator`s. This allows processing data coming from a stream in parallel,
//! enabling use of *all* system resources.
//!
//! You can read about the design decisions and motivation in the "parallel
//! streams" section of the ["streams
//! concurrency"](https://blog.yoshuawuyts.com/streams-concurrency/#parallel-streams)
//! blog post.
//!
//! # Differences with Rayon
//!
//! Rayon is a data parallelism library built for synchronous Rust, powered by
//! an underlying thread pool. async-std manages a thread pool as well, but the
//! key difference with Rayon is that async-std (and futures) are optimized for
//! *latency*, while Rayon is optimized for *throughput*.
//!
//! As a rule of thumb: if you want to speed up doing heavy calculations you
//! probably want to use Rayon. If you want to parallelize network requests
//! consider using `parallel-stream`.
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
//!     assert_eq!(out, vec![1, 4, 9, 16]);
//! }
//! ```

#![forbid(unsafe_code, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples)]

mod from_parallel_stream;
mod from_stream;
mod into_parallel_stream;
mod par_stream;

pub use from_parallel_stream::FromParallelStream;
pub use from_stream::{from_stream, FromStream};
pub use into_parallel_stream::IntoParallelStream;
pub use par_stream::{ForEach, Map, NextFuture, ParallelStream, Take};

pub mod prelude;
pub mod vec;

pub(crate) mod utils;
