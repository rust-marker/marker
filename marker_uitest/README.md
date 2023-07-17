# Marker UI-test

[![Crates.io](https://img.shields.io/crates/v/marker_uitest.svg)](https://crates.io/crates/marker_uitest)
<!--
FIXME(xFrednet): Add license shield, once crates.io also says:
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/marker_uitest.svg)](#license)
-->

The easiest way to test lints, is simply to run them on examples and look at the generated output. We are programmers, which means that we have tools to automate this process. *marker_uitest* is a thin wrapper around the [ui_test] crate for [Marker]. It performs all the common setup magic required to run ui-tests.

[ui_test]: https://crates.io/crates/ui_test
[Marker]: https://github.com/rust-marker/marker

## Prerequisites

*marker_uitest* requires *[Cargo]*, *[rustup]* and [cargo_marker] to be installed.

[Cargo]: https://github.com/rust-lang/cargo/
[rustup]: https://github.com/rust-lang/rustup/
[cargo_marker]: https://crates.io/crates/cargo-marker

## Usage

The [`ui_test`] crate runs Marker on every `.rs`-file in the `tests/ui` folder and compares the output with the `.stderr` and `.stdout` files next to them. To automatically update the `.stderr` and `.stdout` files, you can either run `cargo test -- -- --bless` or set the `RUST_BLESS` environment variable.

For a full list of supported features and magic comments, please refer to the documentation of the [ui_test] crate.

## Setup

<!-- FIXME(xFrednet) Link to the lint-crate template, with the already setup ui tests -->

### Manifest

First add `marker_utils` to the dev-dependencies of the lint crate, and specify that the ui-test doesn't require a test harness, like this:

```toml
[dev-dependencies]
marker_uitest = "<version>"

[[test]]
name = "uitest"
harness = false
```

### Setup test file

Create a `uitest.rs` file in the `tests` directory. Then you can use the following template to get started:

```rust,ignore
use marker_uitest::ui_test::*;
use std::{env, path::Path};

fn main() -> color_eyre::Result<()> {
    let mut config = marker_uitest::simple_ui_test_config!()?;
    
    // Allows you to automatically update `.stderr` and `.stdout` files
    let bless = env::var_os("RUST_BLESS").is_some() || env::args().any(|arg| arg == "--bless");
    if bless {
        config.output_conflict_handling = OutputConflictHandling::Bless
    }

    // Maybe define replacement filters
    config.stderr_filter(r"\\", "/");
    config.stdout_filter(r"\\", "/");

    // Run the test
    run_tests_generic(
        config,
        default_file_filter,
        default_per_file_config,
        status_emitter::Text,
    )
}
```

