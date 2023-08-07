# Marker Adapter 🔌

[![Crates.io](https://img.shields.io/crates/v/marker_adapter.svg)](https://crates.io/crates/marker_adapter)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/marker_adapter.svg)](#license)

This crate provides a common interface for drivers to communicate with lint crates for [Marker]. It does some heavy lifting which would otherwise need to be done by each individual driver. You're welcome to check out [Marker's Readme] if you're interested in the project.

> **Warning**
>
> This crate is not part of Marker's official API, it's only intended to be used by drivers.

[Marker]: https://github.com/rust-marker/marker
[Marker's Readme]: https://github.com/rust-marker/marker/blob/master/README.md

## Functions

### Driver -> Lint Crate Communication

The adapter can load a list of lint crates and send information from the driver to all lint crates. The adapter and API takes care of safe ABI communication between the driver and lint crates.

### Lint Crate -> Driver Communication

Marker's API requires some callbacks from the lint crates into the driver. The adapter can deal with all the ABI conversions, drivers only need to implement the `DriverContext` trait provided by the adapter.

### Creating an adapter instance

An adapter instance can be crated from the environment. For this, the following environment values are read:

* `MARKER_LINT_CRATES`: A semicolon separated list of crate name and absolute path pairs. Each pair is internally separated by a colon.

## Contributing

Contributions are highly appreciated! If you encounter any issues or have suggestions for improvements, please check out [Marker's GitHub repository](https://github.com/rust-marker/marker).

## License

Copyright (c) 2022-2023 Rust-Marker

Rust-marker is distributed under the terms of the MIT license or the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/rust-marker/marker/blob/master/LICENSE-APACHE), [LICENSE-MIT](https://github.com/rust-marker/marker/blob/master/LICENSE-MIT).
