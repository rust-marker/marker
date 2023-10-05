# Cargo Marker

[![Crates.io](https://img.shields.io/crates/v/cargo_marker.svg)](https://crates.io/crates/cargo_marker)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/cargo_marker.svg)](#license)

*cargo_marker* is the CLI tool for [Marker], an experimental linting interface for Rust. It seamlessly integrates with the Rust compiler to provide custom linting capabilities for your Rust projects.

> **Note**
>
> Marker is in the early stages of development, some things are still missing and the API is still unstable.
>
> A list of limitations and planned features can be found in [Marker's Readme].

[Marker]: https://github.com/rust-marker/marker
[Marker's Readme]: https://github.com/rust-marker/marker/blob/master/README.md

## Key Features

* **Simple CLI**: *cargo_marker* does all the heavy lifting for you, making custom code analysis, as simple as a single console command.
* **Seamless Integration**: *cargo_marker* reuses Rust's existing infrastructure for linting, running Marker as part of your workflow is close to the effort needed for its sibling *[Clippy]*.
* **Automatic Lint-Crate Compilation**: *cargo_marker* automatically fetches and builds specified lint crates, streamlining the process of incorporating additional linting rules into your project.
* **User-Friendly Setup**: *cargo_marker* can automatically set up the driver and toolchain, allowing you to focus on writing quality code. (This version will setup rustc's driver for `nightly-2023-08-24`)

[Clippy]: https://github.com/rust-lang/rust-clippy

## Usage

See the installation and usage instructions in the [main Marker repository README][Marker].
Installation and usage instructions are available in [The Marker Book].

[The Marker Book]: https://rust-marker.github.io/marker/book

## Contributing

Contributions are highly appreciated! If you encounter any issues or have suggestions for improvements, please check out [Marker's GitHub repository](https://github.com/rust-marker/marker).

## License

Copyright (c) 2022-2023 Rust-Marker

Rust-marker is distributed under the terms of the MIT license or the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/rust-marker/marker/blob/master/LICENSE-APACHE), [LICENSE-MIT](https://github.com/rust-marker/marker/blob/master/LICENSE-MIT).
