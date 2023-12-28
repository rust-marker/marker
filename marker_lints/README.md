# Marker Lints

[![Crates.io](https://img.shields.io/crates/v/marker_lints.svg)](https://crates.io/crates/marker_lints)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/marker_lints.svg)](#license)

A collection of lints for the development of lint crates with the [marker_api] and [marker_utils] crates.

> **Note**
>
> Marker is in the early stages of development, some things are still missing and the API is still unstable.
>
> A list of limitations and planned features can be found in [Marker's Readme].

[Marker]: https://github.com/rust-marker/marker
[Marker's Readme]: https://github.com/rust-marker/marker/blob/master/README.md
[marker_api]: https://crates.io/crates/marker_api
[marker_utils]: https://crates.io/crates/marker_utils

## Lints:

This crate currently provides the following lints:
* `marker::diag_msg_uppercase_start`

## Usage

To use `marker_lints` in your project, simply add it to your `Cargo.toml` under the `[workspace.metadata.marker.lints]` section. [cargo_marker] will then automatically fetch the crate and include is when running `cargo marker`.

<!-- region replace marker version stable -->
```toml
[workspace.metadata.marker.lints]
marker_lints = "0.5.0"
```
<!-- endregion replace marker version stable -->

If you want to develop something with Marker, you might want to check out the [lint crate template] which already contains everything you need to get started.

[cargo_marker]: https://crates.io/crates/cargo_marker
[lint crate template]: https://github.com/rust-marker/lint-crate-template

## Contributing

Contributions are highly appreciated! If you encounter any issues or have suggestions for improvements, please check out [Marker's GitHub repository](https://github.com/rust-marker/marker).

## License

Copyright (c) 2022-2023 Rust-Marker

Rust-marker is distributed under the terms of the MIT license or the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/rust-marker/marker/blob/master/LICENSE-APACHE), [LICENSE-MIT](https://github.com/rust-marker/marker/blob/master/LICENSE-MIT).
