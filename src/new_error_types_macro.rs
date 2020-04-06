#[macro_export]
macro_rules! new_error_types {
    () => {};

    ($vis:vis struct $name:ident($type:ty); $($tail:tt)*) => {
        $crate::new_error_struct! { $vis struct $name($type); }
        $crate::new_error_types! { $($tail)* }
    };

    ($vis:vis enum $name:ident {$($var_name:ident $var_payload:tt),* $(,)?} $($tail:tt)*) => {
        $crate::new_error_enum! { $vis enum $name {$($var_name $var_payload),*} }
        $crate::new_error_types! { $($tail)* }
    };
}
