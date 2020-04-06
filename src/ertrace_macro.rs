
#[macro_export]
macro_rules! ertrace {
    ($cause:expr => $struct_name:ident) => {{
        let cause_ertrace: $crate::Ertrace = $cause.into();
        let ertrace = $crate::Ertrace::from_cause(cause_ertrace,
            $crate::new_ertrace_location!($struct_name));
        $struct_name(ertrace)
    }};

    ($cause:expr =>) => {{
        {
            let cause_ertrace: &mut $crate::Ertrace = $cause.as_mut();
            cause_ertrace.push_back($crate::new_ertrace_location!(=>));
        }
        $cause        
    }};

    ($struct_name:ident) => {{
        let ertrace = $crate::Ertrace::new($crate::new_ertrace_location!($struct_name));
        $struct_name(ertrace)
    }};
}
