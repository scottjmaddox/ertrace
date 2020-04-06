#[macro_export]
macro_rules! new_error_enum {
    // Exit rules
    (
        @collect_unitary_variants $vis:vis $name:ident
        ($(,)*) -> ($($var_names:ident,)*)
    ) => {
        #[derive(Debug)]
        $vis enum $name {
            $($var_names($var_names)),*
        }

        impl core::convert::From<$name> for $crate::Ertrace {
            fn from(v: $name) -> $crate::Ertrace {
                match v {
                    $(
                        $name::$var_names(err) => {
                            err.into()
                        }
                    )*
                }
            }
        }

        $(
            impl core::convert::From<$var_names> for $name {
                fn from(v: $var_names) -> $name {
                    $name::$var_names(v)
                }
            }
        )*

        impl core::convert::AsMut<$crate::Ertrace> for $name {
            fn as_mut(&mut self) -> &mut $crate::Ertrace {
                match self {
                    $(
                        $name::$var_names(err) => {
                            err.as_mut()
                        }
                    )*
                }
            }
        }

        // impl core::convert::AsRef<$crate::Ertrace> for $name {
        //     fn as_ref(&self) -> &$crate::Ertrace {
        //         use $name::*;
        //         &self.0
        //     }
        // }
    };

    // // Consume an attribute.
    // (
    //     @collect_unitary_variants $vis:vis $name:ident
    //     (#[$_attr:meta] $($tail:tt)*) -> ($($var_names:tt)*)
    // ) => {
    //     new_error_enum! {
    //         @collect_unitary_variants $vis $name
    //         ($($tail)*) -> ($($var_names)*)
    //     }
    // };

    // Handle a variant, optionally with an with initialiser.
    (
        @collect_unitary_variants $vis:vis $name:ident
        ($var:ident $(= $_val:expr)*, $($tail:tt)*) -> ($($var_names:tt)*)
    ) => {
        new_error_enum! {
            @collect_unitary_variants $vis $name
            ($($tail)*) -> ($($var_names)* $var,)
        }
    };

    // Abort on variant with a payload.
    (
        @collect_unitary_variants $vis:vis $name:ident
        ($var:ident $_struct:tt, $($tail:tt)*) -> ($($var_names:tt)*)
    ) => {
        const _error: () = "cannot parse unitary variants from enum with non-unitary variants";
    };
    
    // Entry rule.
    (
        $vis:vis enum $name:ident {$($body:tt)*}
    ) => {
        new_error_enum! {
            @collect_unitary_variants
            $vis $name ($($body)*,) -> ()
        }
    };
}
