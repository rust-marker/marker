//@rustc-env:RUST_BACKTRACE=0
//@normalize-stderr-test: "lib.rs:.*" -> "lib.rs"

// This function will trigger a panic in the `uilints` lint crate.
// I want to see how the ICE message looks and make sure that it
// links to the rust-marker/marker repo.
//
// Note: Panicking across crate bounds is unsound, that's a problem
// we'll be ignoring for now rust-marker/marker#10
fn uilints_please_ice_on_this() {}

fn main() {}
