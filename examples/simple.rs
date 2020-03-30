fn main() {
    errtrace::init(1024);
    errtrace::try_or_fatal!(a());
}

errtrace::new_type!(AError);
fn a() -> Result<(), AError> {
    b().map_err(|e| errtrace::new_from!(AError, e))?;
    Ok(())
}

errtrace::new_type!(BError);
fn b() -> Result<(), BError> {
    c().map_err(|e| errtrace::new_from!(BError, e))?;
    Ok(())
}

errtrace::new_type!(CError);
fn c() -> Result<(), CError> {
    d().map_err(|e| errtrace::new_from!(CError, e))?;
    Ok(())
}

errtrace::new_type!(DError);
fn d() -> Result<(), DError> {
    Err(errtrace::new!(DError))
}
