#[derive(Debug)]
pub struct ErtraceLocation {
    pub tag: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
    pub module_path: &'static str,
}

impl core::fmt::Display for ErtraceLocation {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        writeln!(
            f,
            "{} at {}:{}:{} in {}",
            self.tag, self.file, self.line, self.column, self.module_path
        )
    }
}

//TODO: confirm that the compiler only inserts a single
// static string for file! and module_path!, rather than
// one per invocation.

/// Create `ErtraceLocation` at the given code location
#[macro_export]
macro_rules! new_ertrace_location {
    ($tag:expr) => {{
        static LOC: $crate::ErtraceLocation = $crate::ErtraceLocation {
            tag: core::stringify!($tag),
            file: core::file!(),
            line: core::line!(),
            column: core::column!(),
            module_path: core::module_path!(),
        };
        &LOC
    }};
    (=>) => {{
        static LOC: $crate::ErtraceLocation = $crate::ErtraceLocation {
            tag: "=>",
            file: core::file!(),
            line: core::line!(),
            column: core::column!(),
            module_path: core::module_path!(),
        };
        &LOC
    }};
}
