/// Use local dev build of driver nearby `cargo-marker` executable
pub fn is_local_driver() -> bool {
    std::env::var("MARKER_NO_LOCAL_DRIVER").is_err() && cfg!(debug_assertions)
}
