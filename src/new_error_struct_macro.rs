#[macro_export]
macro_rules! new_error_struct {
    ($vis:vis struct $struct_name:ident;) => {
        // e.g. `new_error_struct!(pub struct AError);`
        #[derive(Debug)]
        $vis struct $struct_name($crate::Ertrace);

        impl core::convert::From<$struct_name> for $crate::Ertrace {
            fn from(v: $struct_name) -> $crate::Ertrace {
                v.0
            }
        }

        impl core::convert::AsMut<$crate::Ertrace> for $struct_name {
            fn as_mut(&mut self) -> &mut $crate::Ertrace {
                &mut self.0
            }
        }

        impl core::convert::AsRef<$crate::Ertrace> for $struct_name {
            fn as_ref(&self) -> &$crate::Ertrace {
                &self.0
            }
        }
    };
}
