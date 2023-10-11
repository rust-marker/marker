# Setting Lint Levels

Once your crate is configured to run some lints, it quickly becomes important to control the lint levels within your code.

Marker provides the ability to use normal lint control attributes `#[allow(...)]`, `#[deny(...)]`, and `#[warn(....)]` to change the behavior of marker lints. One caveat is that you need to put these attributes under a [conditionally-compiled attribute](#conditional-compilation) `#[cfg_attr(marker, ...)]`.

**Example:**

```rust
#[cfg_attr(marker, allow(marker::lint_crate::lint_name))]
fn foo() {}
```

## Lints namespacing

Marker uses the `marker::` tool prefix for lints. This is to make sure that your lints never collide with the [native `rustc` lints](https://doc.rust-lang.org/rustc/lints/listing/index.html) and lints from any other linting tools. This is similar to how `clippy` puts all of its lints under `clippy::` prefix.

After `marker::` there must be the name of the [lint crate](./lint-crate-declaration.md). This is to make sure that lints from different lint crates never collide with each other. The name of the lint crate must be in snake case. The rule is the same as when you reference the crate in your code as a dependency. If the name of the crate contains dashes, they will be replaced with underscores.

The last segment is the name of the lint itself, which is the lowercaed name of the static variable that defines it in the lint crate.

## Conditional compilation

There is a problem that a regular `cargo check/build` knows nothing about Marker and it will complain about unknown lints unless marker-specific attributes are compiled-out. To work around this Marker passes a `--cfg=marker` flag that you can use in your code.

This allows you to conditionally include `allow/warn/deny` attributes such that only `cargo marker` sees them, and regular builds ignore them allowing you to continue using the version of the Rust toolchain you are using in your project.
