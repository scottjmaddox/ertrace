#[macro_export]
macro_rules! new_error_types {
    () => {};

    ($vis:vis struct $struct_name:ident; $($tail:tt)*) => {
        $crate::new_error_struct! { $vis struct $struct_name; }
        $crate::new_error_types! { $($tail)* }
    };

    ($vis:vis enum $name:ident {$($var_name:ident $var_payload:tt),* $(,)?} $($tail:tt)*) => {
        $crate::new_error_enum! { $vis enum $name {$($var_name $var_payload),*} }
        $crate::new_error_types! { $($tail)* }
    };
}
