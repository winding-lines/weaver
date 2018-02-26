use std::ffi::{OsStr, OsString};

/// Create a shell command from the various parts. For now just joins,
/// need to quote special characters next.
#[allow(unused)]
pub fn to_shell_command<T>(args: T) -> OsString
    where T: IntoIterator, T::Item: AsRef<OsStr> {
    let mut out = OsString::new();
    for a in args {
        out.push(a);
    }
    out
}