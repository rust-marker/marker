<h1 align="center">Marker</h1>
<p align="center">
    <br/>
    <img height=200 src="./res/marker-logo.svg" alt="Marker logo" style="border-radius: 50%;"/>
</p>

<br/>

[![Bors enabled](https://bors.tech/images/badge_small.svg)](https://app.bors.tech/repositories/46214)
[![CI Status](https://github.com/rust-marker/marker/actions/workflows/rust_bors.yml/badge.svg)](https://github.com/rust-marker/marker/actions?query=event%3Apush+workflow%3A%22Rust+(bors)%22)

**An experimental linting interface for Rust. Let's make custom lints a reality!**

> **Note**
>
> The project is in the very early stages of development, some things are still missing and the API is not stable yet.

## Status

What works:
- A basic Item and Pattern representation is available. Expressions and types are partially implemented
- Dynamic libraries can be loaded as *lint crates*. A few example lints can be found in the `marker_uitest` directory
- The API is separated from the driver and translation layer. Rustc is currently the only backend development along with the API

What's next:
- The main goal is to complete the AST representation, and start on user tests
- A focus is also set on tooling. Working with marker should be simple and fun for lint developers and users.
- The project is designed to support multiple drivers and unify code analysis for Rust. Additionally, drivers like `rust-analyzer` will be investigated.
- ...and a lot more things! Check out the [open issues](https://github.com/rust-marker/marker/issues?q=is%3Aissue+is%3Aopen+sort%3Aupdated-desc)

Sounds interesting?
* Do you have a lint idea? Help us out by creating a [user story](https://github.com/rust-marker/design/issues/new?assignees=&labels=A-user-story&template=user-story.md&title=)
* Have you found a bug or have suggestions? Please [create an issue](https://github.com/rust-marker/marker/issues/new), any kind of feedback or question is welcome!
* Do you want to help? Even better, check out [open issues](https://github.com/rust-marker/marker/issues?q=is%3Aissue+is%3Aopen+sort%3Aupdated-desc)

## Running

The project is currently only available in this GitHub repo.
For a quick test, clone the repo and run `cargo dogfood`.
This will start `cargo-marker`, load [`./marker_uitest`](./marker_uitest) as a *lint crate* and run it in this repository.

## Contributing

Internal structure of marker and its components is documented in [`docs/internal/`](./docs/internal/).

## License

Copyright (c) 2022-2023 Rust-Marker

Rust-marker is distributed under the terms of the MIT license
or the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/rust-marker/marker/blob/master/LICENSE-APACHE), [LICENSE-MIT](https://github.com/rust-marker/marker/blob/master/LICENSE-MIT).
