use std::ffi::OsString;

#[allow(clippy::unnecessary_wraps)]
pub fn to_os_str(bytes: Vec<u8>) -> Option<OsString> {
    #[cfg(unix)]
    {
        use std::os::unix::prelude::OsStringExt;
        Some(OsString::from_vec(bytes))
    }

    // Windows paths are guaranteed to be valid UTF
    #[cfg(windows)]
    Some(OsString::from(String::from_utf8(bytes).ok()?))
}
