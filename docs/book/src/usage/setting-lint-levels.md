# Setting Lint Levels
Once your crate is configured to run some lints, it quickly becomes important to control the lint levels within your
code. Marker provides the ability to use normal lint control attributes like `#[allow(...)]` `#[deny(...)]` and others
to control the behavior of marker lints.

## On nightly
Marker uses the `marker::` tool prefix for lints. In order for rustc to recognize this prefix, it requires marker to be
registered via the [`#[register_tool()]` feature](https://github.com/rust-lang/rust/issues/66079).

If your crate is compiled using nightly, then controlling lints is as simple as placing `#![feature(register_tool)]`
and `#![register_tool(marker)]` at the top of your crate `lib.rs` or `mod.rs` file. You can then use all of the normal
lint attributes you might usually use, on marker provided lints, like `#[allow(marker::my_lint)]`.


#### Nightly Required Attributes in `lib.rs` / `main.rs`
```rust
#![feature(register_tool)]
#![register_tool(marker)]
```

#### Nightly Lint Attribute
```rust
// Marker lints can be controlled like this
#[allow(marker::my_lint)]
fn foo() {}
```

## On Stable
If your project isn't on nightly, you can still use the `register_tool` attribute, you'll just need to add some extra
guards around it. Marker provides config arguments to rust during the lint passes, so that your linted code can
conditionally apply attributes only during the marker run.

Specifically marker passes `--cfg=marker` and a `--cfg=marker="my_crate"` flag for each lint crate. This means that you
can use `#[cfg_attr(marker, foo)]` to conditionally apply the `foo` attribute only during lint runs.

This conditional compilation can be used to leverage the fact that marker uses nightly for linting, without requiring
the project to use a nightly toolchain. To do this, add `#![cfg_attr(marker, feature(register_tool))]` and
`#![cfg_attr(marker, register_tool(marker))]` attributes to the top of your crate file, to register marker as a tool.
Then you can apply lint level attributes like `#[cfg_attr(marker, allow(marker::foo))]` to control your marker lints.

Additionally, you can check if a specific lint crate is in the set of loaded lint crates. This is useful, when you
only want to enable some attributes if specific lints are loaded. For this you can use a `marker = "<lint-crate-name>"`
check, like this: `#[cfg_attr(marker = "my_crate", allow(marker::foo))]`.

#### Stable Required Attributes in `lib.rs` / `main.rs`
```rust
#![cfg_attr(marker, feature(register_tool))]
#![cfg_attr(marker, register_tool(marker))]
```

#### Stable Conditional Lint Attribute
```rust
// Marker lints can be controlled like this
#[cfg_attr(marker, allow(marker::my_lint))]
fn foo() {}
```
