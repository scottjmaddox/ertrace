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
//! fn a() -> Result<(), AError> {
//!     b().map_err(|e| ertrace!(AError caused by e))?;
//!     Ok(())
//! }
//! new_error_type!(struct AError);
//!
//! fn b() -> Result<(), BError> {
//!     Err(ertrace!(BError))
//! }
//! new_error_type!(struct BError);
//!
//! ertrace::init(1024);
//! ertrace::try_or_fatal!(a());
//! ```
//!
//! Output:
//! ```skip
//! error return trace:
//! 0: BError at src/lib.rs:13:9 in rust_out
//! 1: AError at src/lib.rs:7:21 in rust_out
//! 
//! thread 'main' panicked at 'fatal error', src/lib.rs:18:1
//! note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
//! ```
//!
//! ## Complex Example
//!
//! ```rust,should_panic
//! use ertrace::{ertrace, new_error_type};
//!
//! fn main() {
//!     ertrace::init(1024);
//!     ertrace::try_or_fatal!(a());
//! }
//! 
//! fn a() -> Result<(), AError> {
//!     crate::b::b().map_err(|e| ertrace!(AError caused by e))?;
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
//!                 ertrace!(BError(BErrorKind::BError1) caused by e),
//!             crate::c::CErrorKind::CError2 =>
//!                 ertrace!(BError(BErrorKind::BError2) caused by e),
//!         })?;
//!         Ok(())
//!     }
//!     new_error_type!(pub struct BError(pub BErrorKind));
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
//!     pub enum CErrorKind { CError1, CError2 }
//! }
//! ```
//!
//! Output:
//! ```skip
//! error return trace:
//! 0: CError(CErrorKind::CError1) at src/lib.rs:37:17 in rust_out::c
//! 1: BError(BErrorKind::BError1) at src/lib.rs:22:17 in rust_out::b
//! 2: AError at src/lib.rs:11:31 in rust_out
//! 
//! thread 'main' panicked at 'fatal error', src/lib.rs:7:5
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

#[cfg(feature = "alloc")]
extern crate alloc;

use core::ptr::NonNull;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use static_assertions::const_assert;

//TODO: wrap these in a cache-padded struct?
static TRACE_NODE_ARENA_START: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
static TRACE_NODE_ARENA_OFFSET_MASK: AtomicUsize = AtomicUsize::new(0);
static TRACE_NODE_ARENA_OFFSET_UNMASKED: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct ErrorTraceNode {
    pub location: &'static ErrorTraceLocation,
    pub cause: *mut ErrorTraceNode,
}
const_assert!(core::mem::size_of::<ErrorTraceNode>().is_power_of_two());

#[derive(Debug)]
pub struct ErrorTraceLocation {
    pub tag: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
    pub module_path: &'static str,
}

pub trait GetErrorTraceNodePtr {
    fn get_error_trace_node_ptr(&self) -> NonNull<ErrorTraceNode>;
}

fn align_of_ptr(ptr: *mut u8) -> usize {
    1 << (ptr as usize).trailing_zeros()
}

pub unsafe fn init_from_memory(start: *mut u8, size: usize) {
    assert!(align_of_ptr(start) >= core::mem::align_of::<ErrorTraceNode>());
    assert!(size.is_power_of_two());
    assert!(size <= isize::max_value() as usize);
    assert_eq!(size % core::mem::size_of::<ErrorTraceNode>(), 0);
    assert!(TRACE_NODE_ARENA_START.load(Ordering::SeqCst).is_null());
    TRACE_NODE_ARENA_START.store(start, Ordering::SeqCst);
    TRACE_NODE_ARENA_OFFSET_MASK.store(size - 1, Ordering::SeqCst);
}

#[cfg(feature = "alloc")]
pub fn init(n: usize) {
    let size = n * core::mem::size_of::<ErrorTraceNode>();
    let align = core::mem::align_of::<ErrorTraceNode>();
    let layout = alloc::alloc::Layout::from_size_align(size, align).unwrap();
    let start = unsafe { alloc::alloc::alloc(layout) };
    assert!(!start.is_null());
    unsafe { init_from_memory(start, size) };
}

pub fn alloc_trace_node() -> NonNull<ErrorTraceNode> {
    let start = TRACE_NODE_ARENA_START.load(Ordering::SeqCst);
    assert!(!start.is_null());
    // NOTE: since the arena size and size_of::<ErrorTraceNode>()
    // are both enforced to be powers of two, we can rely on
    // integer addition for wrap-around, rather than needing to do an
    // atomic compare-and-swap after checking for wrap around. We just
    // need to make sure we mask the offset before offsetting into the arena.
    let unmasked_offset = TRACE_NODE_ARENA_OFFSET_UNMASKED
        .fetch_add(core::mem::size_of::<ErrorTraceNode>(), Ordering::SeqCst);
    let offset = unmasked_offset & TRACE_NODE_ARENA_OFFSET_MASK.load(Ordering::SeqCst);
    // NOTE: this cast is safe, because the arena size is enforced to fit in isize.
    let offset = offset as isize;
    unsafe {
        let ptr = start.offset(offset) as *mut ErrorTraceNode;
        NonNull::new_unchecked(ptr)
    }
}

#[macro_export]
macro_rules! new_error_type {
    (struct $struct_name:ident) => {
        // e.g. `new_error_type!(struct Error);`
        struct $struct_name(core::ptr::NonNull<$crate::ErrorTraceNode>);

        impl $crate::GetErrorTraceNodePtr for $struct_name {
            fn get_error_trace_node_ptr(&self) -> core::ptr::NonNull<$crate::ErrorTraceNode> {
                self.0
            }
        }
    };

    (pub struct $struct_name:ident) => {
        // e.g. `new_error_type!(pub struct AError);`
        pub struct $struct_name(core::ptr::NonNull<$crate::ErrorTraceNode>);

        impl $crate::GetErrorTraceNodePtr for $struct_name {
            fn get_error_trace_node_ptr(&self) -> core::ptr::NonNull<$crate::ErrorTraceNode> {
                self.0
            }
        }
    };

    (pub struct $struct_name:ident(pub $enum_name:ident)) => {
        // e.g. `new_error_type!(pub struct BError(pub BErrorKind));`
        pub struct $struct_name(
            pub $enum_name,
            core::ptr::NonNull<$crate::ErrorTraceNode>);

        impl $crate::GetErrorTraceNodePtr for $struct_name {
            fn get_error_trace_node_ptr(&self) -> core::ptr::NonNull<$crate::ErrorTraceNode> {
                self.1
            }
        }
    };
}

#[macro_export]
macro_rules! ertrace {
    ($struct_name:ident($variant:expr) caused by $cause:expr) => ({
        let cause: &dyn $crate::GetErrorTraceNodePtr = &$cause;
        let node = $crate::ErrorTraceNode {
            location: $crate::error_trace_location!($struct_name($variant)),
            cause: cause.get_error_trace_node_ptr().as_ptr(),
        };
        let node_ptr = $crate::alloc_trace_node();
        unsafe {
            core::ptr::write(node_ptr.as_ptr(), node);
        }
        $struct_name($variant, node_ptr)
    });

    ($struct_name:ident caused by $cause:expr) => {{
        let cause: &dyn $crate::GetErrorTraceNodePtr = &$cause;
        let node = $crate::ErrorTraceNode {
            location: $crate::error_trace_location!($struct_name),
            cause: cause.get_error_trace_node_ptr().as_ptr(),
        };
        let node_ptr = $crate::alloc_trace_node();
        unsafe {
            core::ptr::write(node_ptr.as_ptr(), node);
        }
        $struct_name(node_ptr)
    }};

    ($struct_name:ident($variant:expr)) => ({
        let node = $crate::ErrorTraceNode {
            location: $crate::error_trace_location!($struct_name($variant)),
            cause: core::ptr::null_mut(),
        };
        let node_ptr = $crate::alloc_trace_node();
        unsafe {
            core::ptr::write(node_ptr.as_ptr(), node);
        }
        $struct_name($variant, node_ptr)
    });

    ($struct_name:ident) => {{
        let node = $crate::ErrorTraceNode {
            location: $crate::error_trace_location!($struct_name),
            cause: core::ptr::null_mut(),
        };
        let node_ptr = $crate::alloc_trace_node();
        unsafe {
            core::ptr::write(node_ptr.as_ptr(), node);
        }
        $struct_name(node_ptr)
    }};
}

#[macro_export(local_inner_macros)]
/// Create `ErrorTraceLocation` at the given code location
macro_rules! error_trace_location {
    ($tag:expr) => {{
        //TODO: confirm that the compiler only inserts a single
        // static string for file! and module_path!, rather than
        // one per invocation.
        static LOC: $crate::ErrorTraceLocation = $crate::ErrorTraceLocation {
            tag: core::stringify!($tag),
            file: core::file!(),
            line: core::line!(),
            column: core::column!(),
            module_path: core::module_path!(),
        };
        &LOC
    }};
}

pub fn eprint(node_ptr: NonNull<ErrorTraceNode>) {
    let mut nodes = Vec::<ErrorTraceNode>::new();
    let mut node_ptr = node_ptr.as_ptr();
    loop {
        let node = unsafe { core::ptr::read(node_ptr) };
        node_ptr = node.cause;
        nodes.push(node);
        if node_ptr.is_null() {
            // this is the end of the linked list
            break;
        }
    }
    eprintln!("error return trace:");
    for (i, node) in nodes.iter().rev().enumerate() {
        let loc = node.location;
        eprintln!(
            "{}: {} at {}:{}:{} in {}",
            i, loc.tag, loc.file, loc.line, loc.column, loc.module_path
        );
    }
    eprintln!("");
}

#[cfg(feature = "std")]
#[macro_export]
macro_rules! eprint {
    ($err:expr) => {{
        let err: &dyn $crate::GetErrorTraceNodePtr = &$err;
        $crate::eprint(err.get_error_trace_node_ptr());
    }};
}

#[cfg(feature = "std")]
#[macro_export]
macro_rules! fatal {
    ($err:expr) => {{
        $crate::eprint!($err);
        panic!("fatal error");
    }};
}

#[cfg(feature = "std")]
#[macro_export]
macro_rules! try_or_fatal {
    ($e:expr) => {{
        match $e {
            Ok(v) => v,
            Err(err) => {
                $crate::fatal!(err);
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic]
    fn simple() {
        crate::new_error_type!(struct AError);

        fn a() -> Result<(), AError> {
            b().map_err(|e| crate::ertrace!(AError caused by e))?;
            Ok(())
        }

        crate::new_error_type!(struct BError);

        fn b() -> Result<(), BError> {
            Err(crate::ertrace!(BError))
        }

        crate::init(1024);
        crate::try_or_fatal!(a());
    }

    #[test]
    #[should_panic]
    fn variants() {
        fn a() -> Result<(), AError> {
            b().map_err(|e| crate::ertrace!(AError caused by e))?;
            Ok(())
        }
        crate::new_error_type!(struct AError);

        fn b() -> Result<(), BError> {
            c().map_err(|e| match e.0 {
                CErrorKind::CError1 => crate::ertrace!(BError(BErrorKind::BError1) caused by e),
                CErrorKind::CError2 => crate::ertrace!(BError(BErrorKind::BError2) caused by e),
            })?;
            Ok(())
        }
        crate::new_error_type!(pub struct BError(pub BErrorKind));
        pub enum BErrorKind { BError1, BError2 }

        fn c() -> Result<(), CError> {
            if true {
                Err(crate::ertrace!(CError(CErrorKind::CError1)))
            } else {
                Err(crate::ertrace!(CError(CErrorKind::CError2)))
            }
        }
        crate::new_error_type!(pub struct CError(pub CErrorKind));
        pub enum CErrorKind { CError1, CError2 }

        crate::init(1024);
        crate::try_or_fatal!(a());
    }
}
