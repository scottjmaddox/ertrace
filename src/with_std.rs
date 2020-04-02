#[macro_export]
macro_rules! eprint_ertrace {
    ($err:expr) => {{
        let ertrace: &$crate::Ertrace = $err.as_ref();
        eprint!("{}", ertrace);
    }};
}

#[macro_export]
macro_rules! fatal {
    ($err:expr) => {{
        $crate::eprint_ertrace!(&$err);
        panic!("fatal error");
    }};
}

#[macro_export]
macro_rules! try_or_fatal {
    ($e:expr) => {{
        match $e {
            Ok(v) => v,
            Err(err) => {
                $crate::fatal!(err);
            }
        }
    }};
}
