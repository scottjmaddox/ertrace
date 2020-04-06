use ertrace::{ertrace};

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

ertrace::new_error_types! {
    // Define new traced error structs `AError`, `BError1`, and `BError2`.
    pub struct AError;
    pub struct BError1;
    pub struct BError2;

    // Define a new traced error enum `BError`, with variants for
    // `BError1` and `BError2`.
    pub enum BError {
        BError1(BError1),
        BError2(BError2),
    }
}
