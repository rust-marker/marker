# _The Adapter_

The adapter connects the lint driver with lint crates. :electric_plug:

:warning: This crate is not part of the stable API :warning: 

## Interface

The adapter usually gets created from the environment. The following environment values can be used:
* `LINTER_LINT_CRATES`: A semicolon separated list of dynamic libraries belonging to lint crates. Ideally, these should hold absolute paths.

  Example: `/path/to/lint_crates/one.so;/path/to/lint_crates/two.so`

## Implementation

This crate currently loads lint crates as dynamic libraries. The general concept is based on [this article](https://adventures.michaelfbryan.com/posts/plugins-in-rust/). The implementation transfers a trait object over the C ABI, which is kind of undefined. See [`improper_ctypes_definitions`](https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#improper-ctypes-definitions) for a brief explanation. This currently works but might break in the future, this should be changed before `v1.0.0`.
