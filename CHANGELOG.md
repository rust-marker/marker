[Unreleased]: https://github.com/rust-marker/marker/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/rust-marker/marker/releases/tag/v0.3.0
[0.2.1]: https://github.com/rust-marker/marker/releases/tag/v0.2.1
[0.1.1]: https://github.com/rust-marker/marker/releases/tag/v0.1.1

[#174]: https://github.com/rust-marker/marker/issues/174

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2023-09-03

### Other

- More fixes in grammar and wording
- Fix wording
- Fixes from the review
- Fix small typos and grammar mistakes
- Improve error handling and observability


## [0.2.1] - 2023-08-24

See the v0.2.0 milestone for a full list of all changes

### Added
- Support `.await` and `async` expressions
- Started [The Marker Book](https://rust-marker.github.io/marker/book/)

### Fixed

- `cargo marker` now works with lower toolchain versions
- Fixed errors due to drifts in the used toolchain
- Fixed the question mark operator resugar
- `Span`s now properly represent macro expansions

### Changed (breaking)
- `FnItem<'ast>` and `ClosureExpr<'ast>` no longer implement `CallableData`
- Some `Span` methods have been renamed


## [0.1.1] - 2023-07-17

Marker's first release intended for user testing.

### Features
This version is far from done, but it already includes most critical AST nodes required for linting. This is an incomplete list of supported elements:

- Items
- Generics
- Statements
- Expressions
- Patterns
- Types
Marker should be able to handle all stable features, except `async` and `await` expressions. (See: [#174])

### Installation
You can install Marker with cargo, like this:

```bash
cargo install cargo_marker

# Automatically setup the toolchain and driver
cargo marker setup --auto-install-toolchain
```

To run Marker simply specify a lint crate in your `Cargo.toml` file, like this:

```toml
[workspace.metadata.marker.lints]
marker_lints = "0.1.0"
```

And run:

```
cargo marker
```

This should give you the usual `Checking ...` output of Cargo.

### Development
You might want to check out Marker's [lint crate template](https://github.com/rust-marker/lint-crate-template) to test the API yourself.

### Feedback
This release is intended to collect feedback of any kind. If you encounter any bugs, have any thoughts or suggestions, please create an issue or reach out to me directly.

Happy linting everyone! 🦀 🖊️ 🎉
