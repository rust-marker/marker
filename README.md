# Rust linting

A project trying to implement a stable linting API for Rust.

## Contributing

The internal structure of the linter and its component is documented in [`docs/internal/`](./docs/internal/).

## Running

The project is currently only available in this GitHub repo. For a quick test, checkout the repo and run `cargo run -- -l ./linter_test`. This will start `cargo-linter` and load [`./linter_test`](./linter_test) as a *lint crate*.

## License

Copyright (c) 2022-2022 Rust-linting

Rust-linting is distributed under the terms of both the MIT license
and the Apache License (Version 2.0).

See [LICENSE-APACHE](./LICENSE-APACHE), [LICENSE-MIT](./LICENSE-MIT).
