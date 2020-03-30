use ertrace::{ertrace, new_error_type};

fn main() {
    ertrace::init(1024);
    ertrace::try_or_fatal!(a());
}

fn a() -> Result<(), AError> {
    b().map_err(|e| ertrace!(AError caused by e))?;
    Ok(())
}
new_error_type!(struct AError);

fn b() -> Result<(), BError> {
    Err(ertrace!(BError))
}
new_error_type!(struct BError);
