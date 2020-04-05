#[test]
#[should_panic]
fn simple() {
    use crate::{ertrace, new_error_type};
    fn a() -> Result<(), AError> {
        b().map_err(|e| ertrace!(e => AError))?;
        Ok(())
    }
    new_error_type!(struct AError);

    fn b() -> Result<(), BError> {
        b_inner().map_err(|mut e| ertrace!(e =>))
    }
    
    fn b_inner() -> Result<(), BError> {
        Err(ertrace!(BError))
    }
    new_error_type!(struct BError);
    
    #[cfg(feature = "std")]
    crate::try_or_fatal!(a());
    #[cfg(not(feature = "std"))]
    a().unwrap()
}

#[test]
#[should_panic]
fn simple_kinds() {
    use crate::{ertrace, new_error_type};
    fn a() -> Result<(), AError> {
        b().map_err(|e| ertrace!(e => AError))?;
        Ok(())
    }
    new_error_type!(pub struct AError);
    
    fn b() -> Result<(), BError> {
        b_inner().map_err(|mut e| ertrace!(e =>))
    }
    
    fn b_inner() -> Result<(), BError> {
        if true {
            Err(ertrace!(BError(BErrorKind::BError1)))
        } else {
            Err(ertrace!(BError(BErrorKind::BError2)))
        }
    }
    new_error_type!(pub struct BError(pub BErrorKind));
    
    #[derive(Debug)]
    pub enum BErrorKind { BError1, BError2 }
    
    #[cfg(feature = "std")]
    crate::try_or_fatal!(a());
    #[cfg(not(feature = "std"))]
    a().unwrap()
}
