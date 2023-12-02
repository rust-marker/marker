# Marker Utils

[![Crates.io](https://img.shields.io/crates/v/marker_utils.svg)](https://crates.io/crates/marker_utils)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/marker_utils.svg)](#license)

Marker utils aims to be the standard library for the development of lint crates for [Marker], an experimental linting interface for Rust. This crate contains all additional functionality needed to work swiftly with the [marker_api] crate

> **Note**
>
> Marker is in the early stages of development, this crate is very limited, some things are still missing and the API is still unstable.
>
> A list of limitations and planned features can be found in [Marker's Readme].

[Marker]: https://github.com/rust-marker/marker
[Marker's Readme]: https://github.com/rust-marker/marker/blob/master/README.md
[marker_api]: https://crates.io/crates/marker_api

## Usage

To get started, just include *marker_utils* as a dependency:

<!-- region replace marker version stable -->
```toml
[dependencies]
marker_api = "0.4.3"
```

You can also add [marker_lints] as a lint crate, designed for this crate:

```toml
[workspace.metadata.marker.lints]
marker_lints = "0.4.3"
```
<!-- endregion replace marker version stable -->

If you want to develop something with Marker, you might want to check out the [lint crate template] which already contains everything you need to get started.

[lint crate template]: https://github.com/rust-marker/lint-crate-template
[marker_api]: https://crates.io/crates/marker_api
[cargo_marker]: https://crates.io/crates/cargo_marker
[marker_lints]: https://crates.io/crates/marker_lints

## Contributing

Contributions are highly appreciated! If you encounter any issues or have suggestions for improvements, please check out [Marker's GitHub repository](https://github.com/rust-marker/marker).

## License

Copyright (c) 2022-2023 Rust-Marker

Rust-marker is distributed under the terms of the MIT license or the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/rust-marker/marker/blob/master/LICENSE-APACHE), [LICENSE-MIT](https://github.com/rust-marker/marker/blob/master/LICENSE-MIT).
