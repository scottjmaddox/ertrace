#[macro_export]
macro_rules! new_error_enum {
    ($vis:vis enum $name:ident {$($var_name:ident $var_payload:tt),* $(,)?}) => {
        #[derive(Debug)]
        $vis enum $name {
            $(
                $var_name $var_payload,
            )*
        }

        impl core::convert::From<$name> for $crate::Ertrace {
            fn from(v: $name) -> $crate::Ertrace {
                match v {
                    $(
                        $name::$var_name(err) => {
                            err.into()
                        }
                    )*
                }
            }
        }

        $(
            impl core::convert::From<$var_name> for $name {
                fn from(v: $var_name) -> $name {
                    $name::$var_name(v)
                }
            }
        )*

        impl core::convert::AsMut<$crate::Ertrace> for $name {
            fn as_mut(&mut self) -> &mut $crate::Ertrace {
                match self {
                    $(
                        $name::$var_name(err) => {
                            err.as_mut()
                        }
                    )*
                }
            }
        }

        impl core::convert::AsRef<$crate::Ertrace> for $name {
            fn as_ref(&self) -> &$crate::Ertrace {
                match self {
                    $(
                        $name::$var_name(err) => {
                            err.as_ref()
                        }
                    )*
                }
            }
        }
    };
}
