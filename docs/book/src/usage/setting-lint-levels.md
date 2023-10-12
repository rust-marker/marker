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

### Per-lint-crate cfg

Marker also passes `--cfg=marker="lint_crate"` for each loaded lint crate that participates in linting. You may use this flag to conditionally control the level of lints from a specific lint crate.

This is intended to be used only for lints that are time consuming. For example, you could group lints that take a lot of time to run in a separate lint crate. This way you can run Marker with these additional lints enabled on CI, but without them locally.

**Example:**

Run lints from a `heavy_lint_crate` only on CI.
```bash
cargo marker --lints 'heavy_lint_crate = 1.0.0'
```

Use the default lints from `Cargo.toml` metadata locally.
```bash
cargo marker
```

Conditionally ignore the lint from the `heavy_lint_crate`, but only if it actually participates in linting.
```rust
#[cfg_attr(marker = "heavy_lint_crate", allow(marker::heavy_lint_crate::lint_name))]
fn foo() {}
```

This way you'll avoid triggering an "unknown lint" error if the `heavy_lint_crate` isn't included during the linting.

The reason for doing this kind of conditional compilation instead of just including `heavy_lint_crate` in linting unconditionally is a bit intricate. If you put any `allow` attributes in your code for the heavy lints then this won't prevent the them from running. Of course, the lints for code under an `allow` will not be emitted, but the lints logic still walks through that code. Thefore, if you want to prevent a heavy lint from running you should just not include it in the lints configured in `Cargo.toml` or with `--lints` CLI parameter.
