//@rustc-env:RUST_BACKTRACE=0
//@normalize-stderr-test: "rustc 1\..* running on .*" -> "rustc <version> running on <target>"
//@normalize-stderr-test: "(?ms)query stack during panic:\n.*end of query stack\n" -> ""
//@normalize-stderr-test: "marker_rustc_driver .*" -> "marker_rustc_driver bar"

// This function will trigger a panic in rustc's driver.
// I want to see how the ICE message looks and make sure that it
// links to the rust-marker/marker repo.
fn rustc_driver_please_ice_on_this() {}

fn main() {}
