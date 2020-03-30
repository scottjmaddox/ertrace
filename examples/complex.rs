fn main() {
    ertrace::init(1024);
    ertrace::try_or_fatal!(a());
}

fn a() -> Result<(), AError> {
    crate::b::b().map_err(|e| ertrace::trace!(AError caused by e))?;
    Ok(())
}
ertrace::new_error_type!(struct AError);

mod b {
    pub fn b() -> Result<(), BError> {
        crate::c::c().map_err(|e| match e.0 {
            crate::c::CErrorKind::CError1 =>
                ertrace::trace!(BError(BErrorKind::BError1) caused by e),
            crate::c::CErrorKind::CError2 =>
                ertrace::trace!(BError(BErrorKind::BError2) caused by e),
        })?;
        Ok(())
    }
    ertrace::new_error_type!(pub struct BError(pub BErrorKind));
    ertrace::new_error_type!(pub enum BErrorKind { BError1, BError2 });
}

mod c {
    pub fn c() -> Result<(), CError> {
        if true {
            Err(ertrace::trace!(CError(CErrorKind::CError1)))
        } else {
            Err(ertrace::trace!(CError(CErrorKind::CError2)))
        }
    }
    ertrace::new_error_type!(pub struct CError(pub CErrorKind));
    ertrace::new_error_type!(pub enum CErrorKind { CError1, CError2 });
}
