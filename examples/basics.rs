use ertrace::{ertrace, new_error_enum, new_error_struct};

#[cfg(feature = "std")]
fn main() {
    // On any error in `a`, print the error return trace to stderr,
    // and then `panic!`.
    ertrace::try_or_fatal!(a());
}

#[cfg(not(feature = "std"))]
fn main() -> Result<(), AError> {
    a()
}

fn a() -> Result<(), AError> {
    // On any error in `b`, return an `AError`, and trace the cause.
    b().map_err(|e| ertrace!(e => AError))?;
    Ok(())
}
// Define a new traced error type, `AError`.
new_error_struct!(pub struct AError);

fn b() -> Result<(), BError> {
    // Forward any `BError` errors from `b_inner`.
    b_inner().map_err(|mut e| ertrace!(e =>))
}

fn b_inner() -> Result<(), BError> {
    if true {
        // Initialize and return a traced error, `BError`,
        // with error kind `BError1`.
        Err(ertrace!(BError1))?
    } else {
        // Initialize and return a traced error, `BError`,
        // with error kind `BError2`.
        Err(ertrace!(BError2))?
    }
}
// Define new traced error structs `BError1` and `BError2`.
new_error_struct!(pub struct BError1);
new_error_struct!(pub struct BError2);

// Define a new traced error enum `BError`, with variants for
// `BError1` and `BError2`.
new_error_enum!(
    pub enum BError {
        BError1(BError1),
        BError2(BError2),
    }
);

// Define the `BError` error kinds.
#[derive(Debug)]
pub enum BErrorKind {
    BError1,
    BError2,
}
