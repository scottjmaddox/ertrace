#[macro_export]
macro_rules! new_error_enum {
    // (@as_expr $e:expr) => {$e};
    // (@as_item $($i:item)+) => {$($i)+};
    
    // Exit rules.
    // (
    //     @collect_unitary_variants $name:ident //($callback:ident ( $($args:tt)* )),
    //     ($(,)*) -> ($($var_names:ident,)*)
    // ) => {
    //     new_error_enum! {
    //         @as_expr
    //         $vis enum $name {
    //             $($var_names),*
    //         }
    //         //$callback!{ $($args)* ($($var_names),*) }
    //     }
    // };

    (
        @collect_unitary_variants $vis:vis $name:ident //($callback:ident { $($args:tt)* }),
        ($(,)*) -> ($($var_names:ident,)*)
    ) => {
        // new_error_enum! {
        //     @as_item
        //     $vis enum $name {
        //         $($var_names),*
        //     }
        //     //$callback!{ $($args)* ($($var_names),*) }
        // }

        $vis enum $name {
            $($var_names),*
        }
    };

    // Consume an attribute.
    (
        @collect_unitary_variants $vis:vis $name:ident //$fixed:tt,
        (#[$_attr:meta] $($tail:tt)*) -> ($($var_names:tt)*)
    ) => {
        new_error_enum! {
            @collect_unitary_variants $vis $name //$fixed,
            ($($tail)*) -> ($($var_names)*)
        }
    };

    // Handle a variant, optionally with an with initialiser.
    (
        @collect_unitary_variants $vis:vis $name:ident //$fixed:tt,
        ($var:ident $(= $_val:expr)*, $($tail:tt)*) -> ($($var_names:tt)*)
    ) => {
        new_error_enum! {
            @collect_unitary_variants $vis $name //$fixed,
            ($($tail)*) -> ($($var_names)* $var,)
        }
    };

    // Abort on variant with a payload.
    (
        @collect_unitary_variants $vis:vis $name:ident //$fixed:tt,
        ($var:ident $_struct:tt, $($tail:tt)*) -> ($($var_names:tt)*)
    ) => {
        const _error: () = "cannot parse unitary variants from enum with non-unitary variants";
    };
    
    // Entry rule.
    (
        $vis:vis enum $name:ident {$($body:tt)*} //=> $callback:ident $arg:tt
    ) => {
        new_error_enum! {
            @collect_unitary_variants
            $vis $name
            //($callback $arg), 
            ($($body)*,) -> ()
        }
    };
}
