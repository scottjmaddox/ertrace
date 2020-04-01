# Ertrace

**Experimental Error Return Tracing for Rust.**

The immediate goals of this library are to:
    1. provide a minimal-boilerplate error handling story based around error
       return tracing, and
    2. demonstrate the value of error return tracing with the hopes of
       getting support directly integrated into the Rust compiler.

This library is very much in its early days and is highly unstable. Some
effort has been made to implement functionality with performance in mind,
but, so far, no profiling has been performed. There is undoubtedly room for
improvement.

## Error Return Tracing

Error return tracing is a novel error handling concept developed by
Andrew Kelley for the Zig programming language. Error return traces look a
bit like the stack traces displayed by many popular programming languages
when an exception goes uncaught. Stack traces provide extremely valuable
information for identifying the source of an error, but, unfortunately, they
have a considerable performance cost. For this reason, Rust only enables
stack traces for panics, and only when the `RUST_BACKTRACE` environment
variable is defined.

Error return traces provide similar information to stack traces, but at
a far smaller performance cost. They achieve this by tracing errors
as they bubble up the call stack, rather than by capturing an entire
stack trace when an error is first encountered. (For more information on
the performance differences between stack traces and error return traces,
please see the [section on performance](
#performance-stack-traces-vs-error-return-traces), below.)

Furthermore, error return traces can even provide *more* useful information
than basic stack traces, since they trace where and why an error of one type
causes an error of another type. Finally, since the errors are traced
through each return point, error return tracing works seamlessly with
M:N threading, futures, and async/await.

## Simple Example

```rust
use ertrace::{ertrace, new_error_type};

fn a() -> Result<(), AError> {
    b().map_err(|e| ertrace!(AError caused by e))?;
    Ok(())
}
new_error_type!(struct AError);

fn b() -> Result<(), BError> {
    Err(ertrace!(BError))
}
new_error_type!(struct BError);

ertrace::init(1024);
ertrace::try_or_fatal!(a());
```

Output:

```
error return trace:
0: BError at src/lib.rs:13:9 in rust_out
1: AError at src/lib.rs:7:21 in rust_out

thread 'main' panicked at 'fatal error', src/lib.rs:18:1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## Complex Example

```rust
use ertrace::{ertrace, new_error_type};

fn main() {
    ertrace::init(1024);
    ertrace::try_or_fatal!(a());
}

fn a() -> Result<(), AError> {
    crate::b::b().map_err(|e| ertrace!(AError caused by e))?;
    Ok(())
}
new_error_type!(struct AError);

mod b {
    use ertrace::{ertrace, new_error_type};

    pub fn b() -> Result<(), BError> {
        crate::c::c().map_err(|e| match e.0 {
            crate::c::CErrorKind::CError1 =>
                ertrace!(BError(BErrorKind::BError1) caused by e),
            crate::c::CErrorKind::CError2 =>
                ertrace!(BError(BErrorKind::BError2) caused by e),
        })?;
        Ok(())
    }
    new_error_type!(pub struct BError(pub BErrorKind));
    pub enum BErrorKind { BError1, BError2 }
}

mod c {
    use ertrace::{ertrace, new_error_type};

    pub fn c() -> Result<(), CError> {
        if true {
            Err(ertrace!(CError(CErrorKind::CError1)))
        } else {
            Err(ertrace!(CError(CErrorKind::CError2)))
        }
    }
    new_error_type!(pub struct CError(pub CErrorKind));
    pub enum CErrorKind { CError1, CError2 }
}
```

Output:

```
error return trace:
0: CError(CErrorKind::CError1) at src/lib.rs:37:17 in rust_out::c
1: BError(BErrorKind::BError1) at src/lib.rs:22:17 in rust_out::b
2: AError at src/lib.rs:11:31 in rust_out

thread 'main' panicked at 'fatal error', src/lib.rs:7:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```


## `#![no_std]` Support

Ertrace provides `no_std` support.
By default, it depends on the `alloc` and `std` crates, in order
to provide additional functionality, but these dependencies are
gated behind the `alloc` and `std` features, respectively, and can be
disabled by specifying `default-features = false` in your Cargo
dependencies.

## Performance: Stack Traces vs. Error Return Traces

In order for a stack trace to be displayed when an exception goes uncaught,
the entire stack trace must be captured when the exception is created (or
when it is thrown/raised). This is a fairly expensive operation since it
requires traversing each stack frame and storing (at minimum) a pointer to
each function in the call stack in some thread-local storage (which is
typically heap-allocated). The argument usually made is that exceptions
should only be thrown in exceptional cases, and so the performance cost of
collecting a stack trace will not significantly degrade the overall program
performance. In reality, though, errors are quite common, and the cost of
stack traces is not negligible.

In contrast, the cost of error return tracing starts very small, and scales
linearly with the number of times errors are returned. If an error is
handled one stack frame above where it is first created, the overhead
runtime cost can be as small as a few ALU ops and a single memory write.
