#[test]
#[should_panic]
fn basics() {
    use crate::{ertrace, new_error_enum, new_error_struct};
    
    fn a() -> Result<(), AError> {
        b().map_err(|e| ertrace!(e => AError))?;
        Ok(())
    }
    new_error_struct!(pub struct AError);
    
    fn b() -> Result<(), BError> {
        b_inner().map_err(|mut e| ertrace!(e =>))
    }
    
    fn b_inner() -> Result<(), BError> {
        if true {
            Err(ertrace!(BError1))?
        } else {
            Err(ertrace!(BError2))?
        }
    }
    new_error_struct!(pub struct BError1);
    new_error_struct!(pub struct BError2);
    new_error_enum!(pub enum BError {
        BError1(BError1),
        BError2(BError2),
    });
    
    #[cfg(feature = "std")]
    crate::try_or_fatal!(a());
    #[cfg(not(feature = "std"))]
    a().unwrap()
}
