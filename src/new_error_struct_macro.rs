#[macro_export]
macro_rules! new_error_struct {
    (struct $struct_name:ident) => {
        // e.g. `new_error_struct!(struct Error);`
        #[derive(Debug)]
        struct $struct_name($crate::Ertrace);

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

    (pub struct $struct_name:ident) => {
        // e.g. `new_error_struct!(pub struct AError);`
        #[derive(Debug)]
        pub struct $struct_name($crate::Ertrace);

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

    (pub struct $struct_name:ident(pub $enum_name:ident)) => {
        // e.g. `new_error_struct!(pub struct BError(pub BErrorKind));`
        #[derive(Debug)]
        pub struct $struct_name(pub $enum_name, $crate::Ertrace);

        impl core::convert::From<$struct_name> for $crate::Ertrace {
            fn from(v: $struct_name) -> $crate::Ertrace {
                v.1
            }
        }

        impl core::convert::AsMut<$crate::Ertrace> for $struct_name {
            fn as_mut(&mut self) -> &mut $crate::Ertrace {
                &mut self.1
            }
        }

        impl core::convert::AsRef<$crate::Ertrace> for $struct_name {
            fn as_ref(&self) -> &$crate::Ertrace {
                &self.1
            }
        }
    };
}
