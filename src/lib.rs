//! **Experimental Error Return Tracing for Rust.**
//!
//! ## Error Return Tracing
//!
//! TODO
//!
//! ## Example
//!
//! ```rust,should_panic
//! errtrace::new_type!(AError);
//! 
//! fn a() -> Result<(), AError> {
//!     b().map_err(|e| errtrace::new_from!(AError, e))?;
//!     Ok(())
//! }
//! 
//! errtrace::new_type!(BError);
//! 
//! fn b() -> Result<(), BError> {
//!     Err(errtrace::new!(BError))
//! }
//! 
//! errtrace::init(1024);
//! errtrace::try_or_fatal!(a());
//! ```
//!
//! Output:
//! ```skip
//! error return trace:
//! 0: BError at src/lib.rs:12:9 in rust_out
//! 1: AError at src/lib.rs:6:21 in rust_out
//! 
//! thread 'main' panicked at 'fatal error', src/lib.rs:16:1
//! note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
//! ```
//!
//! ## `#![no_std]` Support
//!
//! Errtrace provides `no_std` support.
//! By default, it depends on the `alloc` and `std` crates, in order
//! to provide additional functionality, but these dependencies are
//! gated behind the `alloc` and `std` features, respectively, and can be
//! disabled by specifying `default-features = false` in your Cargo dependencies.

// TODO: uncomment these
// #![deny(missing_debug_implementations)]
// #![deny(missing_docs)]
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

//TODO: wrap these in a cache-padded struct
static TRACE_NODE_ARENA_START: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
static TRACE_NODE_ARENA_OFFSET_MASK: AtomicUsize = AtomicUsize::new(0);
static TRACE_NODE_ARENA_OFFSET_UNMASKED: AtomicUsize = AtomicUsize::new(0);

pub trait GetTraceNodePtr {
    fn get_trace_node_ptr(&self) -> NonNull<TraceNode>;
}

pub struct TraceNode {
    pub location: &'static TraceLocation,
    pub cause: *mut TraceNode,
}
const_assert!(core::mem::size_of::<TraceNode>().is_power_of_two());

pub struct TraceLocation {
    pub err_name: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
    pub module_path: &'static str,
}

fn align_of_ptr(ptr: *mut u8) -> usize {
    1 << (ptr as usize).trailing_zeros()
}

pub unsafe fn init_from_memory(start: *mut u8, size: usize) {
    assert!(align_of_ptr(start) >= core::mem::align_of::<TraceNode>());
    assert!(size.is_power_of_two());
    assert!(size <= isize::max_value() as usize);
    assert_eq!(size % core::mem::size_of::<TraceNode>(), 0);
    assert!(TRACE_NODE_ARENA_START.load(Ordering::SeqCst).is_null());
    TRACE_NODE_ARENA_START.store(start, Ordering::SeqCst);
    TRACE_NODE_ARENA_OFFSET_MASK.store(size - 1, Ordering::SeqCst);
}

#[cfg(feature = "alloc")]
pub fn init(n: usize) {
    let size = n * core::mem::size_of::<TraceNode>();
    let align = core::mem::align_of::<TraceNode>();
    let layout = alloc::alloc::Layout::from_size_align(size, align).unwrap();
    let start = unsafe { alloc::alloc::alloc(layout) };
    assert!(!start.is_null());
    unsafe { init_from_memory(start, size) };
}

pub fn alloc_trace_node() -> NonNull<TraceNode> {
    let start = TRACE_NODE_ARENA_START.load(Ordering::SeqCst);
    assert!(!start.is_null());
    // NOTE: since the arena size and size_of::<TraceNode>()
    // are both enforced to be powers of two, we can rely on
    // integer addition for wrap-around, rather than needing to do an
    // atomic compare-and-swap after checking for wrap around. We just
    // need to make sure we mask the offset before offsetting into the arena.
    let unmasked_offset = TRACE_NODE_ARENA_OFFSET_UNMASKED
        .fetch_add(core::mem::size_of::<TraceNode>(), Ordering::SeqCst);
    let offset = unmasked_offset & TRACE_NODE_ARENA_OFFSET_MASK.load(Ordering::SeqCst);
    // NOTE: this cast is safe, because the arena size is enforced to fit in isize.
    let offset = offset as isize;
    unsafe {
        let ptr = start.offset(offset) as *mut TraceNode;
        NonNull::new_unchecked(ptr)
    }
}

#[macro_export]
macro_rules! new_type {
    ($err_name:ident) => {
        pub struct $err_name(core::ptr::NonNull<$crate::TraceNode>);
        impl $crate::GetTraceNodePtr for $err_name {
            fn get_trace_node_ptr(&self) -> core::ptr::NonNull<$crate::TraceNode> {
                self.0
            }
        }
    };
}

#[macro_export]
macro_rules! new {
    ($err_name:ident) => {{
        let node = $crate::TraceNode {
            location: $crate::trace_location!($err_name),
            cause: core::ptr::null_mut(),
        };
        let node_ptr = $crate::alloc_trace_node();
        unsafe {
            core::ptr::write(node_ptr.as_ptr(), node);
        }
        $err_name(node_ptr)
    }};
}

#[macro_export]
macro_rules! new_from {
    ($err_name:ident, $err_cause:expr) => {{
        let err_cause: &dyn $crate::GetTraceNodePtr = &$err_cause;
        let node = $crate::TraceNode {
            location: $crate::trace_location!($err_name),
            cause: err_cause.get_trace_node_ptr().as_ptr(),
        };
        let node_ptr = $crate::alloc_trace_node();
        unsafe {
            core::ptr::write(node_ptr.as_ptr(), node);
        }
        $err_name(node_ptr)
        // let mem = alloc_trace_node();
        // let node = TraceNode {
        //     location: $crate::trace_location!(),
        //     cause_node_ptr: *mut u8,
        // };
        // let val = $err_name($cause_node_ptr.0);
        // unsafe { *mem = val; }
        // val
    }};
}

#[macro_export(local_inner_macros)]
/// Create `TraceLocation` at the given code location
macro_rules! trace_location {
    ($err_name:ident) => {{
        //TODO: confirm that the compiler only inserts a single
        // static string for file! and module_path!, rather than
        // one per invocation.
        static LOC: $crate::TraceLocation = $crate::TraceLocation {
            err_name: core::stringify!($err_name),
            file: core::file!(),
            line: core::line!(),
            column: core::column!(),
            module_path: core::module_path!(),
        };
        &LOC
    }};
}

pub fn eprint(node_ptr: NonNull<TraceNode>) {
    let mut nodes = Vec::<TraceNode>::new();
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
        eprintln!("{}: {} at {}:{}:{} in {}",
            i, loc.err_name, loc.file, loc.line, loc.column, loc.module_path
        );
    }
    eprintln!("");
}

#[cfg(feature = "std")]
#[macro_export]
macro_rules! eprint {
    ($err:expr) => {{
        let err: &dyn $crate::GetTraceNodePtr = &$err;
        $crate::eprint(err.get_trace_node_ptr());
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
                errtrace::fatal!(err);
            }
        }
}};
}

#[cfg(test)]
mod tests {
}
