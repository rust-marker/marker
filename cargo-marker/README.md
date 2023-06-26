# Cargo Marker

[![Crates.io](https://img.shields.io/crates/v/cargo-marker.svg)](https://crates.io/crates/cargo-marker)
<!--
FIXME(xFrednet): Add license shield, once crates.io also says:
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/cargo_marker.svg)](#license)
-->

*cargo_marker* is the CLI tool for [Marker], an experimental linting interface for Rust. It seamlessly integrates with the Rust compiler to provide custom linting capabilities for your Rust projects.

> **Note**
>
> The project is in the very early stages of development, some things are still missing and the API is not stable yet.
>
> A list of limitations and planned features can be found in [Marker's Readme].

[Marker]: https://github.com/rust-marker/marker
[Marker's Readme]: https://github.com/rust-marker/marker/blob/master/README.md

## Key Features

* **Simple CLI**: *cargo_marker* does all the heavy lifting for you, making custom code analysis, as simple as a single console command.
* **Seamless Integration**: *cargo_marker* reuses Rust's existing infrastructure for linting, running Marker as part of your workflow is close to the effort needed for its sibling *[Clippy]*.
* **Automatic Lint-Crate Compilation**: *cargo_marker* automatically fetches and builds specified lint crates, streamlining the process of incorporating additional linting rules into your project.
* **User-Friendly Setup**: *cargo_marker* can automatically set up the driver and toolchain, allowing you to focus on writing quality code. (This version will setup rustc's driver for `nightly-2023-06-01`)

## Prerequisites

*cargo_marker* requires *[Cargo]* and *[rustup]* to be installed. Currently, only Unix and Windows systems are supported. Linux, Windows, and macOS are actively tested in the CI.

[Cargo]: https://github.com/rust-lang/cargo/
[rustup]: https://github.com/rust-lang/rustup/

## Usage

### Installation

```sh
cargo install cargo_marker

# Automatically setup the toolchain and driver
cargo marker setup --auto-install-toolchain
```

### Specifying lints

Marker requires lint crates to be specified. The best way, is to add them to the `Cargo.toml` file, like this:

```sh
[workspace.metadata.marker.lints]
# Add a local crate as a path
local_lint_crate = { path = "path/to/lint_crate" }
# Add an external crate via git
git_lint_crate = { git = "https://github.com/rust-marker/marker" }
```

Lints from registries, like crates.io, are sadly not yet supported. See [rust-marker/marker#87](https://github.com/rust-marker/marker/issues/87).

### Running Marker

Running Marker is as simple as running its sibling *[Clippy]*. Navigate to your Rust project directory and run the following command:

```sh
cargo marker
```

This will initialize Marker, compile the lint crates and start linting.

[Clippy]: https://github.com/rust-lang/rust-clippy

## Contributing

Contributions are highly appreciated! If you encounter any issues or have suggestions for improvements, please don't hesitate to open an issue or submit a pull request on [Marker's GitHub repository](https://github.com/rust-marker/marker).

## License

Copyright (c) 2022-2023 Rust-Marker

Rust-marker is distributed under the terms of the MIT license
or the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/rust-marker/marker/blob/master/LICENSE-APACHE), [LICENSE-MIT](https://github.com/rust-marker/marker/blob/master/LICENSE-MIT).

