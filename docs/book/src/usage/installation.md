# Installation

Marker is split into several components. This section covers the installation of the `cargo marker` sub-command and the installation of a driver, which is the backend needed to parse the source code.

<!-- toc -->

## Prerequisite

The marker sub-command is provided by the *cargo_marker* crate. This crate requires *[Cargo]* and *[rustup]* to be installed. Currently, only Unix and Windows systems are supported. Ubuntu, Windows, and macOS are actively tested in the CI.

[Cargo]: https://github.com/rust-lang/cargo/
[rustup]: https://github.com/rust-lang/rustup/

## Download pre-compiled binaries (recommended)

We provide pre-compiled binaries for the mainstream platforms. See the list of available artifacts in our [Github Releases](https://github.com/rust-marker/marker/releases/latest).

Run the following bash script to install the required rust toolchain dependency on your machine and download the current version of `cargo-marker` CLI, and the internal driver.

<!-- region replace-version stable -->
```bash
curl --location --silent --fail --show-error --retry 5 --retry-connrefused --retry-delay 30 \
    https://raw.githubusercontent.com/rust-marker/marker/v0.2.1/install.sh | bash
```
<!-- endregion replace-version stable -->

The provided scripts are pinned to a specific version of `marker` to avoid sudden breakages especially if this script will be used on CI.

If you are a windows user or your platform isn't supported yet by the pre-compiled binaries, then you should fall back to building from sources as described below.

## Build `cargo marker` plugin from sources

Marker provides a new Cargo sub-command, that handles the driver installation, lint crate compilation, and checking process for you. To install it, simply use:

```sh
cargo install cargo_marker
```

You now have the new `cargo marker` command installed.

## Driver and Toolchain

Marker requires a driver, which handles the lexing, parsing, and type checking to then translates everything into Marker's API representation to be checked by the lint crates.

### Automatic Installation

Marker's rustc driver requires a specific nightly toolchain to be installed. The nightly version is updated every six weeks when a new version of Rust and Marker is released. You, as the user, should not notice this requirement once it's installed, as `cargo marker` handles everything for you. To automatically install the driver and toolchain, you can simply run the following command:

```sh
cargo marker setup --auto-install-toolchain
```

## Manual Installation

It's highly recommended to use the [*automatic installation*](#automatic-installation) method described above. This is a guide for cases where this is not possible for one reason or another. Note that the manual installation is not actively tested. If you encounter any issues, you're welcome to report them.

### Toolchain

The rustc driver requires a specific nightly version to be built and started. The specific nightly version can be found in the `README.md` of the driver you want to install. The compilation requires at least the following components: `rustc`, `rustc-dev`, `llvm-tools`. These can be installed for the specific toolchain with the following rustup command:

```sh
# Fill in the $toolchain variable
rustup toolchain install $toolchain --component rustc-dev llvm-tools
```

### Driver Compilation

The build script of the driver has a check to prevent accidental compilation of the driver. To manually compile the driver, the `MARKER_ALLOW_DRIVER_BUILD` environment value has to be set. This simply shows the driver that this is not accidental and disables the check. Then it can be compiled as usual. Here is the cargo command:

```sh
# Make sure to run cargo with the correct toolchain.
MARKER_ALLOW_DRIVER_BUILD=1 cargo build --release
```

### Driver Location

By default, the driver is stored with the toolchain that it was built with. This makes it simple to find the right driver for a given toolchain and allows the installation of multiple drivers for different toolchains. You can find the toolchain folder using the following rustup command:

```sh
# Fill in the $toolchain variable
rustup +$toolchain which cargo
```

The driver should be located next to the cargo binary, whose path was given by the previous command. If you're not using rustup, you can store the driver binary next to the `cargo-marker` file. If you're invoking the driver directly, you have to make sure to provide the required libraries to run the driver.

## Driver Selection

The `cargo marker` command searches several locations for the driver and selects the first one it finds. The following locations are searched:
1. The toolchain that was used for the `cargo marker` command.
2. The toolchain that is hard coded in the `cargo-marker` binary. (Updated every six weeks with a new release of the driver and `cargo_marker` crate)
3. Any driver stored next to the `cargo-marker` binary file.
