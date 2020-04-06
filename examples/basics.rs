use ertrace::{ertrace, Ertrace};

fn main() -> Result<(), AError> {
    // Forward any `AError` errors from `a`.
    a().map_err(|mut e| ertrace!(e =>))
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
    pub struct AError(Ertrace);
    pub struct BError1(Ertrace);
    pub struct BError2(Ertrace);

    // Define a new traced error enum `BError`, with variants for
    // `BError1` and `BError2`.
    pub enum BError {
        BError1(BError1),
        BError2(BError2),
    }
}
