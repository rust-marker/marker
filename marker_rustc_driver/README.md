# Marker's driver for rustc

[![Crates.io](https://img.shields.io/crates/v/marker_rustc_driver.svg)](https://crates.io/crates/marker_rustc_driver)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/marker_rustc_driver.svg)](#license)

The rustc driver for [Marker], an experimental linting interface for Rust. This crate wraps around rustc, translates everything into Marker's AST representation, and passes everything to registered lint crates.

> **Warning**
>
> This crate is not intended to be installed manually, please use [cargo_marker] instead. This driver is also not part of Marker's official API. The project is also still just in the early stages of development, some things are still missing from the API and driver.
>
> A list of limitations and planned features can be found in [Marker's Readme].

[Marker]: https://github.com/rust-marker/marker
[Marker's Readme]: https://github.com/rust-marker/marker/blob/master/README.md
[cargo_marker]: https://crates.io/crates/cargo_marker

## Toolchain

<!-- region replace rust toolchain release -->
The driver is linked to a specific nightly rust toolchain. The crate will be updated about every six weeks with a new release of Rust. This version of the driver has been developed for: `nightly-2023-08-24`
<!-- endregion replace rust toolchain release -->

## Contributing

Contributions are highly appreciated! If you encounter any issues or have suggestions for improvements, please check out [Marker's GitHub repository](https://github.com/rust-marker/marker).

## License

Copyright (c) 2022-2023 Rust-Marker

Rust-marker is distributed under the terms of the MIT license or the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/rust-marker/marker/blob/master/LICENSE-APACHE), [LICENSE-MIT](https://github.com/rust-marker/marker/blob/master/LICENSE-MIT).
