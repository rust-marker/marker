# Setting Lint Levels
Once your crate is configured to run some lints, it quickly becomes important to control the lint levels within your
code. Marker provides the ability to use normal lint control attributes like `#[allow(...)]` `#[deny(...)]` and others
to control the behavior of marker lints.

Marker uses the `marker::` tool prefix for lints. You can then use all of the normal lint attributes you might usually use, on marker provided lints, like `#[allow(marker::my_lint)]`, but you have to put them under `#[cfg_attr(marker, ...)]`. This is because normally `rustc` doesn't know anything about `marker` and it will complain about an unknown lint if yo u run simple `cargo check/build`.

Marker provides config arguments to `rustc` during the lint passes, so that your linted code can conditionally apply attributes only during the marker run.

Specifically marker passes `--cfg=marker` and a `--cfg=marker="my_crate"` flag for each lint crate. This means that you
can use `#[cfg_attr(marker, foo)]` to conditionally apply the `foo` attribute only during lint runs.

This conditional compilation can be used to leverage the fact that marker uses a custom rustc implementation, without requiring the project to use that implementation by default. Then you can apply lint level attributes like `#[cfg_attr(marker, allow(marker::foo))]` to control your marker lints.

Additionally, you can check if a specific lint crate is in the set of loaded lint crates. This is useful, when you
only want to enable some attributes if specific lints are loaded. For this you can use a `marker = "<lint-crate-name>"`
check, like this: `#[cfg_attr(marker = "my_crate", allow(marker::foo))]`.

#### Conditional Lint Attribute
```rust
// Marker lints can be controlled like this
#[cfg_attr(marker, allow(marker::my_lint))]
fn foo() {}
```
