#[test]
#[should_panic]
fn basics() {
    use crate::{ertrace, Ertrace};
    
    fn a() -> Result<(), AError> {
        b().map_err(|e| ertrace!(e => AError))
    }
    
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

    crate::new_error_types! {
        pub struct AError(Ertrace);
        pub struct BError1(Ertrace);
        pub struct BError2(Ertrace);
        pub enum BError {
            BError1(BError1),
            BError2(BError2),
        }
    }
    
    #[cfg(feature = "std")]
    crate::try_or_fatal!(a());
    #[cfg(not(feature = "std"))]
    a().unwrap()
}
