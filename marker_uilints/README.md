# Maker's UI tests

UI tests for Marker, implemented as a lint crate.

This is also a playground to experiment with markers API.

## Usage

```sh
# Running these UI tests
cargo uilints

# Blessing the `.stderr`, `.stdout`, `.fixed` files
cargo uilints -- -- --bless
```

## Links:

Please check out the following links for more information about ui-tests:
* [`ui_test` crate](https://crates.io/crates/ui_test)
* [Rustc's dev guide about *UI Tests*](https://rustc-dev-guide.rust-lang.org/tests/ui.html)

