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

## Usage

<!-- Please keep this section in sync with the main readme -->

The following is an abbreviated guide. Check out [The Marker Book] for detailed instructions and additional information.

[The Marker Book]: rust-marker.github.io/marker/book

### Installation

#### Download pre-compiled binaries (recommended)

We provide pre-compiled binaries for the mainstream platforms. See the list of available artifacts in our [Github Releases](https://github.com/rust-marker/marker/releases/latest).

Select one of the installation scripts below according to your platform. The script will install the required Rust toolchain dependency on your machine, download the current version of `cargo-marker` CLI, and the internal driver.

<!-- region replace-version stable -->

**Linux or MacOS (Bash)**:
```bash
curl -fsSL https://raw.githubusercontent.com/rust-marker/marker/v0.2.1/scripts/release/install.sh | bash
```

**Windows (PowerShell)**:
```ps1
curl.exe -fsSL https://raw.githubusercontent.com/rust-marker/marker/v0.2.1/scripts/release/install.ps1 | powershell -command -
```

<!-- endregion replace-version stable -->

The provided scripts are pinned to a specific version of `marker` to avoid sudden breakages especially if this script will be used on CI.

If you are on a platform that isn't supported yet by the pre-compiled binaries, then you should fall back to building from sources as described below.

#### Build from sources

```sh
cargo install cargo_marker

# Automatically setup the toolchain and build driver from sources
cargo marker setup --auto-install-toolchain
```


### Specifying lints

Marker requires lint crates to be specified. The best way is to add them to the `Cargo.toml` file, like this:

<!-- region replace-version stable -->
```toml
[workspace.metadata.marker.lints]
# A local crate as a path
marker_lints = { path = './marker_lints' }
# An external crate via git
marker_lints = { git = "https://github.com/rust-marker/marker" }
# An external crate from a registry
marker_lints = "0.2.1"
```
<!-- endregion replace-version stable -->

### Running Marker

Running Marker is as simple as running its sibling *[Clippy]*. Navigate to your Rust project directory and run the following command:

```sh
cargo marker
```

This will initialize Marker, compile the lint crates and start linting.

[Clippy]: https://github.com/rust-lang/rust-clippy

## Contributing

Contributions are highly appreciated! If you encounter any issues or have suggestions for improvements, please check out [Marker's GitHub repository](https://github.com/rust-marker/marker).

## License

Copyright (c) 2022-2023 Rust-Marker

Rust-marker is distributed under the terms of the MIT license or the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/rust-marker/marker/blob/master/LICENSE-APACHE), [LICENSE-MIT](https://github.com/rust-marker/marker/blob/master/LICENSE-MIT).
