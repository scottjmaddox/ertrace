use ertrace::{ertrace, new_error_type};

fn main() {
    ertrace::try_or_fatal!(a());
}

fn a() -> Result<(), AError> {
    b().map_err(|e| ertrace!(e => AError))?;
    Ok(())
}
new_error_type!(struct AError);

fn b() -> Result<(), BError> {
    Err(ertrace!(BError))
}
new_error_type!(struct BError);
