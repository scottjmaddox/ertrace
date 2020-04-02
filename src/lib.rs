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
//! ## Simple Example
//!
//! ```rust,should_panic
//! use ertrace::{ertrace, new_error_type};
//!
//! fn main() {
//!     ertrace::try_or_fatal!(a());
//! }
//!
//! fn a() -> Result<(), AError> {
//!     b().map_err(|e| ertrace!(e => AError))?;
//!     Ok(())
//! }
//! new_error_type!(struct AError);
//!
//! fn b() -> Result<(), BError> {
//!     Err(ertrace!(BError))
//! }
//! new_error_type!(struct BError);
//! ```
//!
//! Output:
//! ```skip
//! error return trace:
//!     0: BError at src/lib.rs:16:9 in rust_out
//!     1: AError at src/lib.rs:10:21 in rust_out
//! 
//! thread 'main' panicked at 'fatal error', src/lib.rs:6:5
//! note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
//! ```
//!
//! ## Complex Example
//!
//! ```rust,should_panic
//! use ertrace::{ertrace, new_error_type};
//!
//! fn main() {
//!     ertrace::try_or_fatal!(a());
//! }
//!
//! fn a() -> Result<(), AError> {
//!     crate::b::b().map_err(|e| ertrace!(e => AError))?;
//!     Ok(())
//! }
//! new_error_type!(struct AError);
//!
//! mod b {
//!     use ertrace::{ertrace, new_error_type};
//!
//!     pub fn b() -> Result<(), BError> {
//!         crate::c::c().map_err(|e| match e.0 {
//!             crate::c::CErrorKind::CError1 =>
//!                 ertrace!(e => BError(BErrorKind::BError1)),
//!             crate::c::CErrorKind::CError2 =>
//!                 ertrace!(e => BError(BErrorKind::BError2)),
//!         })?;
//!         Ok(())
//!     }
//!     new_error_type!(pub struct BError(pub BErrorKind));
//!     #[derive(Debug)]
//!     pub enum BErrorKind { BError1, BError2 }
//! }
//!
//! mod c {
//!     use ertrace::{ertrace, new_error_type};
//!
//!     pub fn c() -> Result<(), CError> {
//!         if true {
//!             Err(ertrace!(CError(CErrorKind::CError1)))
//!         } else {
//!             Err(ertrace!(CError(CErrorKind::CError2)))
//!         }
//!     }
//!     new_error_type!(pub struct CError(pub CErrorKind));
//!     #[derive(Debug)]
//!     pub enum CErrorKind { CError1, CError2 }
//! }
//! ```
//!
//! Output:
//! ```skip
//! error return trace:
//!     0: CError at src/lib.rs:37:17 in rust_out::c
//!     1: BError at src/lib.rs:21:17 in rust_out::b
//!     2: AError at src/lib.rs:10:31 in rust_out
//! 
//! thread 'main' panicked at 'fatal error', src/lib.rs:6:5
//! note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
//! ```
//!
//!
//! ## `#![no_std]` Support
//!
//! Ertrace provides `no_std` support.
//! By default, it depends on the `alloc` and `std` crates, in order
//! to provide additional functionality, but these dependencies are
//! gated behind the `alloc` and `std` features, respectively, and can be
//! disabled by specifying `default-features = false` in your Cargo
//! dependencies.
//!
//! ## Performance: Stack Traces vs. Error Return Traces
//!
//! In order for a stack trace to be displayed when an exception goes uncaught,
//! the entire stack trace must be captured when the exception is created (or
//! when it is thrown/raised). This is a fairly expensive operation since it
//! requires traversing each stack frame and storing (at minimum) a pointer to
//! each function in the call stack in some thread-local storage (which is
//! typically heap-allocated). The argument usually made is that exceptions
//! should only be thrown in exceptional cases, and so the performance cost of
//! collecting a stack trace will not significantly degrade the overall program
//! performance. In reality, though, errors are quite common, and the cost of
//! stack traces is not negligible.
//!
//! In contrast, the cost of error return tracing starts very small, and scales
//! linearly with the number of times errors are returned. If an error is
//! handled one stack frame above where it is first created, the overhead
//! runtime cost can be as small as a few ALU ops and a single memory write.

#![deny(missing_debug_implementations)]
// #![deny(missing_docs)] // TODO: uncomment this
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;
#[cfg(feature = "std")]
mod with_std;
#[cfg(feature = "std")]
pub use with_std::*;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
mod with_alloc;
#[cfg(feature = "alloc")]
pub use with_alloc::*;
#[cfg(not(feature = "alloc"))]
pub use without_alloc::*;

mod ertrace;
mod ertrace_location;

pub use crate::ertrace::*;
pub use crate::ertrace_location::*;

#[macro_export]
macro_rules! new_error_type {
    (struct $struct_name:ident) => {
        // e.g. `new_error_type!(struct Error);`
        #[derive(Debug)]
        struct $struct_name($crate::Ertrace);

        impl core::convert::From<$struct_name> for $crate::Ertrace {
            fn from(v: $struct_name) -> $crate::Ertrace {
                v.0
            }
        }

        impl core::convert::AsRef<$crate::Ertrace> for $struct_name {
            fn as_ref(&self) -> &$crate::Ertrace {
                &self.0
            }
        }
    };

    (pub struct $struct_name:ident) => {
        // e.g. `new_error_type!(pub struct AError);`
        #[derive(Debug)]
        pub struct $struct_name($crate::Ertrace);

        impl core::convert::From<$struct_name> for $crate::Ertrace {
            fn from(v: $struct_name) -> $crate::Ertrace {
                v.0
            }
        }

        impl core::convert::AsRef<$crate::Ertrace> for $struct_name {
            fn as_ref(&self) -> &$crate::Ertrace {
                &self.0
            }
        }

        impl core::convert::AsRef<$crate::Ertrace> for &$struct_name {
            fn as_ref(&self) -> &$crate::Ertrace {
                &self.0
            }
        }
    };

    (pub struct $struct_name:ident(pub $enum_name:ident)) => {
        // e.g. `new_error_type!(pub struct BError(pub BErrorKind));`
        #[derive(Debug)]
        pub struct $struct_name(pub $enum_name, $crate::Ertrace);

        impl core::convert::From<$struct_name> for $crate::Ertrace {
            fn from(v: $struct_name) -> $crate::Ertrace {
                v.1
            }
        }

        impl core::convert::AsRef<$crate::Ertrace> for $struct_name {
            fn as_ref(&self) -> &$crate::Ertrace {
                &self.1
            }
        }
    };
}

#[macro_export]
macro_rules! ertrace {
    ($cause:expr => $struct_name:ident($variant:expr)) => {{
        let cause_ertrace: $crate::Ertrace = $cause.into();
        let ertrace = $crate::Ertrace::from_cause(cause_ertrace,
            $crate::new_ertrace_location!($struct_name));
        $struct_name($variant, ertrace)
    }};

    ($cause:expr => $struct_name:ident) => {{
        let cause_ertrace: $crate::Ertrace = $cause.into();
        let ertrace = $crate::Ertrace::from_cause(cause_ertrace,
            $crate::new_ertrace_location!($struct_name));
        $struct_name(ertrace)
    }};

    ($struct_name:ident($variant:expr)) => {{
        let ertrace = $crate::Ertrace::new($crate::new_ertrace_location!($struct_name));
        $struct_name($variant, ertrace)
    }};

    ($struct_name:ident) => {{
        let ertrace = $crate::Ertrace::new($crate::new_ertrace_location!($struct_name));
        $struct_name(ertrace)
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic]
    fn simple() {
        crate::new_error_type!(struct AError);

        fn a() -> Result<(), AError> {
            b().map_err(|e| crate::ertrace!(e => AError))?;
            Ok(())
        }

        crate::new_error_type!(struct BError);

        fn b() -> Result<(), BError> {
            Err(crate::ertrace!(BError))
        }

        crate::try_or_fatal!(a());
    }

    #[test]
    #[should_panic]
    fn variants() {
        fn a() -> Result<(), AError> {
            b().map_err(|e| crate::ertrace!(e => AError))?;
            Ok(())
        }
        crate::new_error_type!(struct AError);

        fn b() -> Result<(), BError> {
            c().map_err(|e| match e.0 {
                CErrorKind::CError1 => crate::ertrace!(e => BError(BErrorKind::BError1)),
                CErrorKind::CError2 => crate::ertrace!(e=> BError(BErrorKind::BError2)),
            })?;
            Ok(())
        }
        crate::new_error_type!(pub struct BError(pub BErrorKind));
        #[derive(Debug)]
        pub enum BErrorKind {
            BError1,
            BError2,
        }

        fn c() -> Result<(), CError> {
            if true {
                Err(crate::ertrace!(CError(CErrorKind::CError1)))
            } else {
                Err(crate::ertrace!(CError(CErrorKind::CError2)))
            }
        }
        crate::new_error_type!(pub struct CError(pub CErrorKind));
        #[derive(Debug)]
        pub enum CErrorKind {
            CError1,
            CError2,
        }

        crate::try_or_fatal!(a());
    }
}
