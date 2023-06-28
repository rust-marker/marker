<h1 align="center">Marker</h1>
<p align="center">
    <img height=200 src="./res/marker-logo-round.svg" alt="Marker logo" />
</p>

<br/>

[![Crates.io](https://img.shields.io/crates/v/marker_api.svg)](https://crates.io/crates/marker_api)
<!--
FIXME(xFrednet): Add license shield, once crates.io also says:
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/marker_api.svg)](#license)
-->

Marker is an experimental code analysis interface, with the goal to create a stable and user-friendly linting framework for the Rust programming language. Creating custom lints should be a straightforward process, and using them should be as simple as declaring a new dependency.

Let's make custom lints and code analysis a reality!

> **Note**
>
> The project is in the early stages of development, some things are still missing and the API is not stable yet.
>
> A collection of limitations, goals and features is available below.

## Features

* **Custom Lints**: Marker provides a framework for everyone, to create and provide custom lints. Allowing you to automatically improve the code quality for your project and users of your crate.
* **User-Friendly Interface**: Marker provides a new subcommand for [Cargo] that does all the heavy lifting for you. [cargo_marker] can automatically set up a driver for linting, compile lint crate and run them on any project.
* **Driver independent**: Every code analysis requires a driver that parses the code and provides further information. Marker's API is designed to be driver-independent. Allowing it to support future compilers and potentially IDEs, therefore binging custom lints everywhere. (Currently, only [rustc] is supported)

And more to come, see Marker's goals below.

[Cargo]: https://github.com/rust-lang/cargo/
<!-- FIXME(xFrednet): Update the link to link to `crates.io` once the first version was released-->
[cargo_marker]: https://github.com/rust-marker/marker/tree/master/cargo-marker
[rustc]: https://github.com/rust-lang/rustc/

## Goals

* **Stability**: Marker's API design heavily focuses on stability. Any lint, written after version 1.0.0, should compile and continue to work for years to come.
* **Usability**: Marker's API focuses on usability, where possible under the constraints of Marker's stability guarantees. Types follow common design patterns and naming conventions, allowing you to focus on the lint logic directly.
* **Be A Foundation**: Marker want's to be a foundation, for new tools which make linting even easier, 
provide deeper code analysis capabilities and allow for automated migrations.

A small side note to be clear: Marker doesn't want to replace [Clippy], instead it wants to be the proud sibling of [Clippy]. Changing Rust one lint message at a time.

[Clippy]: https://github.com/rust-lang/rust-clippy

## Usage

To install Marker and run lint crates, please refer to the documentation of [cargo_marker]. For lint development, please check out the documentation of the [marker_api] crate.

<!-- FIXME(xFrednet): Update the link to link to `crates.io` once the first version was released-->
[marker_api]: https://github.com/rust-marker/marker/tree/master/marker_api

## Limitations

Marker is still growing up, and that's a good thing. We can still shape the API and adapt it to what the user needs. However, this and the fact that Marker is not an official Rust project comes with some limitations:

* **Nightly**: Internally, Marker has to rely on nightly versions of rustc. However, you, as an end-user, should not notice this dependency.
* **Lint Crates Registry**: Lint creates can currently only be compiled from paths and git repos. Fetching lint crates from [crates.io] is planned, but not yet supported.
* **AST Limitations**: Marker's API is still missing a few elements to represent everything required for linting. The API is still incomplete when it comes to:
    * `async` expressions
    * Higher order types
    * Array types
    * Attributes
    * Macros
* **Utility**: The API mostly focussed on representing everything in Rust. Next, the API needs to be extended with more utility functions, to make life easier for everyone.
* **Documentation**: Marker still requires a lot of documentation, in the form of doc comments and a book, which explains the basic concept and works as a guide for end-users, lint- and marker-devs alike.

[crates.io]: https://crates.io/

## Timeline

1. Complete a draft of the API.
2. Improve documentation and conduct first user tests.
3. Update and expand the API to incorporate feedback.
4. Reach out to the wider community for more feedback.

The used nightly version will be updated every 6 weeks, when a new version of Rust is released.

## Contributing

Contributions are highly appreciated! If you encounter any issues or have suggestions for improvements, please don't hesitate to [open an issue]. If you have an idea for a lint you want to implement with Marker, please share it by creating a [user story].

Still reading? It would be wonderful if you're interested in helping out! Checkout Marker's [open issues] or [create a new one] to get suggestions for good first issues.

[open an issue]: https://github.com/rust-marker/marker/issues/new
[create a new one]: https://github.com/rust-marker/marker/issues/new
[user story]: https://github.com/rust-marker/design/issues/new?assignees=&labels=A-user-story&template=user-story.md&title=
[open issues]: https://github.com/rust-marker/marker/issues?q=is%3Aissue+is%3Aopen+sort%3Aupdated-desc

## License

Copyright (c) 2022-2023 Rust-Marker

Rust-marker is distributed under the terms of the MIT license or the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/rust-marker/marker/blob/master/LICENSE-APACHE), [LICENSE-MIT](https://github.com/rust-marker/marker/blob/master/LICENSE-MIT).

