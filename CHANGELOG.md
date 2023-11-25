[Unreleased]: https://github.com/rust-marker/marker/compare/v0.4.0...HEAD
[0.4.2]: https://github.com/rust-marker/marker/releases/tag/v0.4.2
[0.4.1]: https://github.com/rust-marker/marker/releases/tag/v0.4.1
[0.4.0]: https://github.com/rust-marker/marker/releases/tag/v0.4.0
[0.3.0]: https://github.com/rust-marker/marker/releases/tag/v0.3.0
[0.2.1]: https://github.com/rust-marker/marker/releases/tag/v0.2.1
[0.1.1]: https://github.com/rust-marker/marker/releases/tag/v0.1.1

# Changelog

⚠️ Marker is in an early stage of development, so there are breaking changes on each `0.x` minor version. Our target is to stabilize all APIs before we reach a `1.0.0`.

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

This project adheres to a stricter subset of [Semantic Versioning](https://semver.org/spec/v2.0.0.html) as described in [cargo's crates versioning](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-cratesio):

> This compatibility convention is different from SemVer in the way it treats versions before 1.0.0. While SemVer says there is no compatibility before 1.0.0, Cargo considers 0.x.y to be compatible with 0.x.z, where y ≥ z and x > 0.

The following components are considered to be internal and they are excluded from the semver and API/ABI stability guarantees.

- `marker_rustc_driver`
- `marker_adapter`
- `marker_error`

## [Unreleased]

[#322]: https://github.com/rust-marker/marker/pull/322

### Added

- [#322]: `sem::TyKind` now implements `From<*>` for all semantic types.

### Breaking Changes

- [#322]: Renamed `sem::TyKind::FnTy` -> `sem::TyKind::Fn`
- [#322]: Renamed `sem::TyKind::ClosureTy` -> `sem::TyKind::Closure`

## [0.4.2] - 2023-11-25

[#320]: https://github.com/rust-marker/marker/pull/320

### Fixed

- [#320]: Disable LTO on release builds to fix the crash on Windows with `exit code: 0xc0000005, STATUS_ACCESS_VIOLATION`

## [0.4.1] - 2023-11-24

[#319]: https://github.com/rust-marker/marker/pull/319

### Fixed

- [#319]: Fix compiling driver from sources outside of the marker repo on Windows


## [0.4.0] - 2023-11-16

The [v0.4.0 milestone] contains a full list of all changes.

[v0.4.0 milestone]: https://github.com/rust-marker/marker/milestone/4
[#278]: https://github.com/rust-marker/marker/pull/278
[#288]: https://github.com/rust-marker/marker/pull/288
[#294]: https://github.com/rust-marker/marker/pull/294
[#306]: https://github.com/rust-marker/marker/pull/306
[#309]: https://github.com/rust-marker/marker/pull/309

### Added
- [#306]: The `LintPass` trait now as a new `check_crate` method.
- [#294]: Items and fields now provide information about their visibility

### Fixed
- [#306]: Rustc's driver no longer ICEs on spans from compiler generated code.

### Breaking Changes
- [#278]: Moved the `span()` method from the inherent impl to `HasSpan` trait for `ClosureParam`, `FnParam`, `StructFieldPat`.
- [#278]: The trait `TyData` no longer has own `span()` method, but instead requires `HasSpan` as a supertrait.
- [#278]: All public methods that took `&Span` as a parameter now take `impl HasSpan`. This is a minor breaking change, as `&Span` implements `HasSpan`, but if you relied on type inference based on the function parameter type, then making a method generic may break your code.
- [#288]: Lint identifiers use the lint crate name as a new infix, to prevent name collisions across lint crates.
- [#306]: The items of a `Crate` are now wrapped in a `ModItem`, that is the root module of the crate.
- [#309]: Renamed `CallExpr::operand` to `CallExpr::func`
- [#309]: Renamed `RefPat::pattern()` -> `RefPat::pat()`
- [#309]: Renamed `OrPat::patterns()` -> `RefPat::pats()`

## [0.3.0] - 2023-10-05

This version introduces precompiled binaries and CI templates. This version also tried to combine all breaking changes, to make `v0.3.0` a solid baseline for new additions.

The [v0.3.0 milestone] contains a full list of all changes.

[v0.3.0 milestone]: https://github.com/rust-marker/marker/milestone/3?closed=1
[#231]: https://github.com/rust-marker/marker/pull/231
[#232]: https://github.com/rust-marker/marker/pull/232
[#239]: https://github.com/rust-marker/marker/pull/239
[#241]: https://github.com/rust-marker/marker/pull/241
[#244]: https://github.com/rust-marker/marker/pull/244
[#245]: https://github.com/rust-marker/marker/pull/245
[#252]: https://github.com/rust-marker/marker/pull/252
[#256]: https://github.com/rust-marker/marker/pull/256
[#259]: https://github.com/rust-marker/marker/pull/259
[#260]: https://github.com/rust-marker/marker/pull/260
[#263]: https://github.com/rust-marker/marker/pull/263
[#265]: https://github.com/rust-marker/marker/pull/265
[#268]: https://github.com/rust-marker/marker/pull/268

### Added
- [#232]: Add scope config for visitors and `for_each_expr` to `marker_utils`
- [#239]: GitHub releases now provide precompiled binaries of `cargo-marker` and `marker_rustc_driver`.
- [#252]: Marker now provides install scripts for linux, macos and windows
- [#259]: Introduced a GitHub Action for installing and running Marker

### Breaking Changes
- [#256]: Renamed `AstContext` -> `MarkerContext`
- [#256]: Moved `marker_api::ast::common::span` -> `marker_api::span`
- [#241]: Renamed `QuestionMarkExpr` -> `TryExpr`
- [#244]: `StmtKind` and `PatKind` no longer wrap `Kind*` directly
- [#245]: `emit_lint()` takes less arguments and returns a `DiagnosticBuilder` instance
- [#263]: Updated the [`ui_test`](https://crates.io/crates/ui_test) used by `marker_uitest` from `v0.11.7` to `v0.21.2`
- [#260]: Moved `AstContext::{body, item, lint_level_at}` to the new `AstMap` struct accessible via `MarkerContext::ast()`
- [#265]: Removed the `CallableData` trait
- [#268]: Moved semantic types and generics to the new `marker_api::sem` module
- [#268]: Moved common items, like IDs, to the new `marker_api::common` module
- [#268]: Removed the `Sem` and `Syn` prefix from types and generics
- [#268]: `marker_api::prelude` no longer contains the semantic and syntactic `TyKind` enums
- [#268]: `marker_api::prelude` now imports the `sem` and `ast` names
- [#268]: The `marker_api::ast` module has been flattened
- [#268]: The `marker_api::lint` and `marker_api::interface` are now private

### Internal

- [#231]: Significantly improved error handling
- [#239]: The release flow was automated. It's now a process of updating the `CHANGELOG.md` and doing several clicks to trigger the CI job.

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

### Breaking Changes
- `FnItem<'ast>` and `ClosureExpr<'ast>` no longer implement `CallableData`
- Some `Span` methods have been renamed


## [0.1.1] - 2023-07-17

[#174]: https://github.com/rust-marker/marker/issues/174

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
