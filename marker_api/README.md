# Marker API

[![Crates.io](https://img.shields.io/crates/v/marker_api.svg)](https://crates.io/crates/marker_api)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/marker_api.svg)](#license)

*marker_api* provides a representation of the AST and all connected types needed to create custom *lint crates* for [Marker], an experimental linting interface for Rust.

> **Note**
>
> Marker is in the early stages of development, some things are still missing and the API still unstable.
>
> A list of limitations and planned features can be found in [Marker's Readme].

[Marker]: https://github.com/rust-marker/marker
[Marker's Readme]: https://github.com/rust-marker/marker/blob/master/README.md

## Goals

* **Stability**: Marker's API design focuses on stability and expendability. The goal is to archive backwards compatibility, so that any lint, written after version 1.0.0, will compile and continue to work for years to come.
* **Usability**: Marker's API focuses on usability, where possible under the constraints of Marker's stability guarantees. Types follow common design patterns and naming conventions, allowing you to focus on the lint logic directly.
* **Driver Independent**: Every code analysis requires a driver that parses the code and provides further information. Marker's API is designed to be driver-independent, allowing it to support future compilers and potentially IDEs. (Currently, [rustc] is the only available driver)

[rustc]: https://github.com/rust-lang/rust/

## Usage

This section will cover how you can set up your own *lint crate*. If you only want to run custom lints, checkout Marker's CLI interface [cargo_marker]. The rest of the section assumes that you have [cargo_marker] installed.

[cargo_marker]: https://crates.io/crates/cargo_marker

### Template

The simplest way to get started, is to use Marker's [lint crate template], which already includes all dependencies, example code, and a working test setup.

[lint crate template]: https://github.com/rust-marker/lint-crate-template

### Manual Setup

#### Cargo.toml

To get started, create a new Rust crate that compiles to a library (`cargo init --lib`). Afterwards, edit the `Cargo.toml` to compile the crate to a dynamic library and include `marker_api` as a dependency. You can simply add the following to your `Cargo.toml` file:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
marker_api = "<version>"
marker_utils = "<version>"
```

#### src/lib.rs

The lint crate needs to provide an implementation of the `LintPass` trait and call the `marker_api::export_lint_pass` macro with the implementing type. Here is the minimal template:

```rust,ignore
use marker_api::prelude::*;
use marker_api::{LintPass, LintPassInfo, LintPassInfoBuilder};

// This is the struct that will implement the `LintPass` trait.
#[derive(Default)]
struct MyLintPass;

// This macro allow Marker to load the lint crate. Only one lint pass can be
// exported per lint crate.
marker_api::export_lint_pass!(MyLintPass);

// This macro declares a new lint, that can later be emitted
marker_api::declare_lint! {
    /// # What it does
    /// Here you can explain what your lint does. The description supports normal
    /// markdown.
    ///
    /// # Example
    /// ```rs
    /// // Bad example
    /// ```
    ///
    /// Use instead:
    /// ```rs
    /// // Good example
    /// ```
    MY_LINT,
    Warn,
}

// This is the actual `LintPass` implementation, which will be called by Marker.
impl LintPass for MyLintPass {
    fn info(&self) -> LintPassInfo {
        LintPassInfoBuilder::new(Box::new([MY_LINT])).build()
    }
}
```

Now you can implement different `check_*` function in the `LintPass` trait.

#### UI-Tests

To automatically test your lints, you might want to check out the [marker_uitest] crate.

And that's it. Happy linting!

[marker_uitest]: https://crates.io/crates/marker_uitest

## Contributing

Contributions are highly appreciated! If you encounter any issues or have suggestions for improvements, please don't hesitate to open an issue or submit a pull request on [Marker's GitHub repository](https://github.com/rust-marker/marker).

## License

Copyright (c) 2022-2023 Rust-Marker

Rust-Marker is distributed under the terms of the MIT license or the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/rust-marker/marker/blob/master/LICENSE-APACHE), [LICENSE-MIT](https://github.com/rust-marker/marker/blob/master/LICENSE-MIT).
