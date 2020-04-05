//! **Experimental Error Return Tracing for Rust.**
//!
//! The immediate goals of this library are to:
//!     1. provide a minimal-boilerplate error handling story based around error
//!        return tracing, and
//!     2. demonstrate the value of error return tracing with the hopes of
//!        getting support directly integrated into the Rust compiler.
//!
//! This library is very much in its early days and is highly unstable. Some
//! effort has been made to implement functionality with performance in mind,
//! but, so far, no profiling has been performed. There is undoubtedly room for
//! improvement.
//!
//! ## Error Return Tracing
//!
//! Error return tracing is a novel error handling concept developed by
//! Andrew Kelley for the Zig programming language. Error return traces look a
//! bit like the stack traces displayed by many popular programming languages
//! when an exception goes uncaught. Stack traces provide extremely valuable
//! information for identifying the source of an error, but, unfortunately, they
//! have a considerable performance cost. For this reason, Rust only enables
//! stack traces for panics, and only when the `RUST_BACKTRACE` environment
//! variable is defined.
//!
//! Error return traces provide similar information to stack traces, but at
//! a far smaller performance cost. They achieve this by tracing errors
//! as they bubble up the call stack, rather than by capturing an entire
//! stack trace when an error is first encountered. (For more information on
//! the performance differences between stack traces and error return traces,
//! please see the [section on performance](
//! #performance-stack-traces-vs-error-return-traces), below.)
//!
//! Furthermore, error return traces can even provide *more* useful information
//! than basic stack traces, since they trace where and why an error of one type
//! causes an error of another type. Finally, since the errors are traced
//! through each return point, error return tracing works seamlessly with
//! M:N threading, futures, and async/await.
//!
//! ## Example
//!
//! ```rust,should_panic
//! use ertrace::{ertrace, new_error_type};
//!
//! fn main() {
//!     // On any error in `a`, print the error return trace to stderr,
//!     // and then `panic!`.
//!     # #[cfg(feature = "std")]
//!     ertrace::try_or_fatal!(a());
//!     # #[cfg(not(feature = "std"))]
//!     # a().unwrap();
//! }
//!
//! fn a() -> Result<(), AError> {
//!     // On any error in `b`, return an `AError`, and trace the cause.
//!     b().map_err(|e| ertrace!(e => AError))?;
//!     Ok(())
//! }
//! // Define a new traced error type, `AError`.
//! new_error_type!(pub struct AError);
//!
//! fn b() -> Result<(), BError> {
//!     // Forward any `BError` errors from `b_inner`.
//!     b_inner().map_err(|mut e| ertrace!(e =>))
//! }
//!
//! fn b_inner() -> Result<(), BError> {
//!     if true {
//!         // Initialize and return a traced error, `BError`,
//!         // with error variant `BError1`.
//!         Err(ertrace!(BError(BErrorKind::BError1)))
//!     } else {
//!         // Initialize and return a traced error, `BError`,
//!         // with error variant `BError2`.
//!         Err(ertrace!(BError(BErrorKind::BError2)))
//!     }
//! }
//! // Define a new traced error type, `BError`, with variant `BErrorKind`.
//! new_error_type!(pub struct BError(pub BErrorKind));
//!
//! // Define the `BError` variants.
//! #[derive(Debug)]
//! pub enum BErrorKind { BError1, BError2 }
//! ```
//!
//! Output:
//! ```skip
//! error return trace:
//!     0: BError(BErrorKind::BError1) at src/lib.rs:28:13 in rust_out
//!     1: => at src/lib.rs:21:31 in rust_out
//!     2: AError at src/lib.rs:13:21 in rust_out
//!
//! thread 'main' panicked at 'fatal error', src/lib.rs:8:5
//! note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
//! ```
//!
//! ## `no_std` Support
//! 
//! Ertrace provides `no_std` support. By default, it depends on the `std` crate,
//! in order to provide additional functionality, such as printing to stderr, but
//! this dependency is gated behind the `std` feature, and can be disabled by
//! specifying `default-features = false` in your Cargo dependencies.
//! 
//! Currently, the `alloc` crate is required, but it should be straight-forward to
//! remove even that requirement by specifying a static block of memory in which
//! to store error traces. If you have a use for this,
//! [please open a Github issue](https://github.com/scottjmaddox/ertrace/issues/new).
//! 
//! ## Performance: Stack Traces vs. Error Return Traces
//! 
//! In order for a stack trace to be displayed when an exception goes uncaught, the
//! entire stack trace must be captured when the exception is created (or when it is
//! thrown/raised). This is a fairly expensive operation since it requires
//! traversing each stack frame and storing (at minimum) a pointer for each function
//! in the call stack, typically in some heap-allocated thread-local storage. The
//! argument usually made is that exceptions should only be thrown in exceptional
//! cases, and so the performance cost of collecting a stack trace will not
//! significantly degrade the overall program performance. In reality, though,
//! errors are quite common, and the cost of stack traces is not negligible.
//! 
//! In contrast, the cost of error return tracing starts very small, and scales
//! linearly with the number of times errors are returned. If an error is
//! handled one stack frame above where it is first created, the overhead
//! runtime cost can be as small as a few ALU ops and a single memory write
//! (if you have compiler support... The runtime overhead for this library
//! implementation is a bit higher).

#![deny(missing_debug_implementations)]
// #![deny(missing_docs)] // TODO: uncomment this
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;
#[cfg(feature = "std")]
mod with_std;
#[cfg(feature = "std")]
pub use with_std::*;

mod ertrace;
mod ertrace_location;
mod ertrace_macro;
mod new_error_type_macro;
#[cfg(test)]
mod tests;

pub use crate::ertrace::*;
pub use crate::ertrace_location::*;
pub use crate::ertrace_macro::*;
pub use crate::new_error_type_macro::*;
