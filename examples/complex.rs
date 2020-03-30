fn main() {
    ertrace::init(1024);
    ertrace::try_or_fatal!(a());
}

ertrace::new_error_type!(struct AError);
fn a() -> Result<(), AError> {
    crate::b::b().map_err(|e| ertrace::trace!(AError from e))?;
    Ok(())
}

mod b {
    ertrace::new_error_type!(pub struct BError);
    // ertrace::new_error_type!(pub enum BError { BError1, BError2 } );
    pub fn b() -> Result<(), BError> {
        crate::c::c().map_err(|e| ertrace::trace!(BError from e))?;
        Ok(())
    }
}

mod c {
    ertrace::new_error_type!(pub struct CError);
    pub fn c() -> Result<(), CError> {
        crate::d::d().map_err(|e| ertrace::trace!(CError from e))?;
        Ok(())
    }
}

mod d {
    ertrace::new_error_type!(pub struct DError);
    pub fn d() -> Result<(), DError> {
        Err(ertrace::trace!(DError))
    }
}
