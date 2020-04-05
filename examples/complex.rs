use ertrace::{ertrace, new_error_struct};

#[cfg(feature = "std")]
fn main() {
    ertrace::try_or_fatal!(a()); 
}

#[cfg(not(feature = "std"))]    
fn main() -> Result<(), AError> {
    a()
}

fn a() -> Result<(), AError> {
    crate::b::b().map_err(|e| ertrace!(e => AError))?;
    Ok(())
}
new_error_struct!(struct AError);

mod b {
    use ertrace::{ertrace, new_error_struct};
    
    pub fn b() -> Result<(), BError> {
        crate::c::c().map_err(|e| match e.0 {
            crate::c::CErrorKind::CError1 =>
                ertrace!(e => BError(BErrorKind::BError1)),
            crate::c::CErrorKind::CError2 =>
                ertrace!(e => BError(BErrorKind::BError2)),
        })?;
        Ok(())
    }
    new_error_struct!(pub struct BError(pub BErrorKind));
    #[derive(Debug)]
    pub enum BErrorKind { BError1, BError2 }
}

mod c {
    use ertrace::{ertrace, new_error_struct};

    pub fn c() -> Result<(), CError> {
        if true {
            Err(ertrace!(CError(CErrorKind::CError1)))
        } else {
            Err(ertrace!(CError(CErrorKind::CError2)))
        }
    }
    new_error_struct!(pub struct CError(pub CErrorKind));
    #[derive(Debug)]
    pub enum CErrorKind { CError1, CError2 }
}
