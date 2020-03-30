fn main() {
    ertrace::init(1024);
    ertrace::try_or_fatal!(a());
}

fn a() -> Result<(), AError> {
    b().map_err(|e| ertrace::trace!(AError caused by e))?;
    Ok(())
}
ertrace::new_error_type!(struct AError);

fn b() -> Result<(), BError> {
    Err(ertrace::trace!(BError))
}
ertrace::new_error_type!(struct BError);
