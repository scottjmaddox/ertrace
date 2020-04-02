use ertrace::{ertrace, new_error_type};

fn main() {
    // On any error in `a`, print the error return trace to stderr,
    // and then `panic!`.
    ertrace::try_or_fatal!(a());
}

fn a() -> Result<(), AError> {
    // On any error in `b`, return an `AError`, and trace the cause.
    b().map_err(|e| ertrace!(e => AError))?;
    Ok(())
}
// Define a new traced error type, `AError`.
new_error_type!(struct AError);

fn b() -> Result<(), BError> {
    // Forward any `BError` errors from `b_inner`.
    b_inner().map_err(|mut e| ertrace!(e =>))
}

fn b_inner() -> Result<(), BError> {
    // Initialize and return a traced error, `BError`.
    Err(ertrace!(BError))
}
// Define a new traced error type, `BError`.
new_error_type!(struct BError);
