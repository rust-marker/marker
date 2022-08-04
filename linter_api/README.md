# Linter API

This crate provides the stable API for lint crates. Here you can find representations for a simple variable name up to the AST of an entire crate. This is the only dependency needed to get start with a new *linting crate*.

:warning: This crate also contains some unstable items, which are required by the current infrastructure. These items are clearly marked and hidden from the documentation. :warning:

## Getting started

To get started, create a new cargo project that compiles to a library (`cargo init --lib`). Afterwards, the `Cargo.toml` has to be edited to compile the crate to a dynamic library. You can simply add the following after the `[package]` values:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
linter_api = "<version>"
```

Now that everything is setup, we jump into `src/lib.rs` where we add everything needed for the linter to load the crate:

```rust,ignore
use linter_api::{lint::Lint, LintPass};

// With the [`declare_lint!`] macro we define a new lint. The macro accepts a
// name, default lint level and description.
linter_api::lint::declare_lint!(YOUR_LINT_NAME, Allow, "the lint descritpion");

// Here we create an object that'll implement `LintPass<'ast>`. This struct can
// hold data used for linting. A mutable reference of this struct is passed to
// each check in `LintPass<'ast>`
#[derive(Debug)]
struct TestLintPass;

// Here we implement the `LintPass` for our struct. It only requires the
// implementation of one function, that returns all lints which are implemented
// by this crate.
impl<'ast> LintPass<'ast> for TestLintPass {
    fn registered_lints(&self) -> Box<[&'static Lint]> {
        Box::new([YOUR_LINT_NAME])
    }

    // Here we can finally start linting, by implementing `check_*` functions.
    // See `LintPass<'ast>` for a complete list of provided callbacks.
}

// Last but not least, we have to mark our object that implements `LintPass`.
// Each lint crate requires exactly one marker. All lints have to be implemented
// in one lint pass. For multiple lints it can be helpful to extract the individual
// linting logic called by the `check_*` functions into separate module.
linter_api::interface::export_lint_pass!(TestLintPass);
```
