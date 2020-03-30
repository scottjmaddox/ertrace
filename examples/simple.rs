fn main() {
    ertrace::init(1024);
    ertrace::try_or_fatal!(a());
}

ertrace::new_error_type!(struct AError);
fn a() -> Result<(), AError> {
    b().map_err(|e| ertrace::trace!(AError from e))?;
    Ok(())
}

ertrace::new_error_type!(struct BError);
fn b() -> Result<(), BError> {
    c().map_err(|e| ertrace::trace!(BError from e))?;
    Ok(())
}

ertrace::new_error_type!(struct CError);
fn c() -> Result<(), CError> {
    d().map_err(|e| ertrace::trace!(CError from e))?;
    Ok(())
}

ertrace::new_error_type!(struct DError);
fn d() -> Result<(), DError> {
    Err(ertrace::trace!(DError))
}
