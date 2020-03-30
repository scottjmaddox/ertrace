use ertrace::{ertrace, new_error_type};

fn main() {
    ertrace::init(1024);
    ertrace::try_or_fatal!(a());
}

fn a() -> Result<(), AError> {
    crate::b::b().map_err(|e| ertrace!(AError caused by e))?;
    Ok(())
}
new_error_type!(struct AError);

mod b {
    use ertrace::{ertrace, new_error_type};
    
    pub fn b() -> Result<(), BError> {
        crate::c::c().map_err(|e| match e.0 {
            crate::c::CErrorKind::CError1 =>
                ertrace!(BError(BErrorKind::BError1) caused by e),
            crate::c::CErrorKind::CError2 =>
                ertrace!(BError(BErrorKind::BError2) caused by e),
        })?;
        Ok(())
    }
    new_error_type!(pub struct BError(pub BErrorKind));
    new_error_type!(pub enum BErrorKind { BError1, BError2 });
}

mod c {
    use ertrace::{ertrace, new_error_type};

    pub fn c() -> Result<(), CError> {
        if true {
            Err(ertrace!(CError(CErrorKind::CError1)))
        } else {
            Err(ertrace!(CError(CErrorKind::CError2)))
        }
    }
    new_error_type!(pub struct CError(pub CErrorKind));
    new_error_type!(pub enum CErrorKind { CError1, CError2 });
}
