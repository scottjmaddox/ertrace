#[macro_export]
macro_rules! new_error_struct {
    ($vis:vis struct $name:ident($type:ty);) => {
        // e.g. `new_error_struct!(pub struct AError);`
        $vis struct $name($type);

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.debug_struct(stringify!($name)).finish()?;
                // writeln!()
                writeln!(f, "\n{}", self.0)
            }
        }

        impl core::convert::From<$name> for $crate::Ertrace {
            fn from(v: $name) -> $crate::Ertrace {
                v.0
            }
        }

        impl core::convert::AsMut<$crate::Ertrace> for $name {
            fn as_mut(&mut self) -> &mut $crate::Ertrace {
                &mut self.0
            }
        }

        impl core::convert::AsRef<$crate::Ertrace> for $name {
            fn as_ref(&self) -> &$crate::Ertrace {
                &self.0
            }
        }
    };
}
