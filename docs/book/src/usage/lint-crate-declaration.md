# Lint Crate Declaration

Marker in itself, is a linting interface. The actual code analysis is implemented in external crates compiled to dynamic libraries, called *lint crates*. This section covers how these lint crates can be specified. Once they're specified, they will be automatically fetched and build by the `cargo marker` command, to then be loaded by the driver for linting.

<!-- toc -->

## Declaration in Cargo.toml

The main way to declare lint crates, is to add them to the `Cargo.toml` file under the `[workspace.metadata.marker.lints]` section. There they can be defined like a normal dependency, with a version, git repository, or path. This is a short example of the three methods:

<!-- region replace marker version stable -->
```toml
[workspace.metadata.marker.lints]
# An external crate from a registry
marker_lints = "0.5.0"

# An external crate from git
marker_lints = { git = "https://github.com/rust-marker/marker" }

# A local crate as a path
marker_lints = { path = './marker_lints' }
```
<!-- endregion replace marker version stable -->

## Declaration as arguments

Lints can also be declared as arguments to the `cargo marker` command. Marker will skip reading the `Cargo.toml` file if any lint crate was specified this way. This is intentional, to allow tools to use Marker for lexing and parsing, regardless of the normally specified lint crates.

A lint crate can be specified with the `--lints` option. The string is expected to have the same format, that would be used in the `Cargo.toml` file. Here is an example for the same lint crates specified above:

<!-- region replace marker version stable -->
```sh
# An external crate from a registry
cargo marker --lints "marker_lints = '0.5.0'"

# An external crate from git
cargo marker --lints "marker_lints = { git = 'https://github.com/rust-marker/marker' }"

# A local crate as a path
cargo marker --lints "marker_lints = { path = './marker_lints' }"
```
<!-- endregion replace marker version stable -->
