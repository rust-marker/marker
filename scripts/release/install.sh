#!/usr/bin/env bash
#
# This script downloads the `cargo-marker` and the `marker_rustc_driver`
# binaries from the GitHub release assets. It also sets up the required
# Rust toolchain that the `marker_rustc_driver` depends on.
#
# This script must be self-contained! Nothing here should use anything from
# the marker repository, because users are expected to run this script on
# their machines where they don't have the marker repository cloned.
#
# This script is specifically for unix, but has a similar structure to the
# windows `install.ps1` script. If you modify this script, please check if the modifications
# should also apply to the windows one.

set -euo pipefail

# This script isn't meant to be run from `master`, but if it is, then
# it will install the latest version be it a stable version or a pre-release.
# region replace-version unstable
version=0.3.0
# endregion replace-version unstable

toolchain=nightly-2023-10-05

function step {
    local cmd="$1"
    shift
    echo -e "\033[32;1m❱ \033[1;33m$cmd\033[0m $@" >&2
    $cmd "$@"
}

echo "Bash version: $BASH_VERSION" >&2

step curl --version
step grep --version
step tar --version

step rustup install --profile minimal --no-self-update $toolchain

# MacOS uses some shitty BSD grep by default, and there isn't support for
# `--perl-regexp` option there, so we have to go without using that.
# Example of such envs are `macos-11` and `macos-12` Github Actions runners.
host_triple=$(\
    rustc "+$toolchain" --version --verbose \
    | grep --only-matching 'host: .*' \
    | grep --only-matching '[^ ]*$'
)

trap cleanup SIGINT SIGTERM ERR EXIT

temp_dir=$(mktemp -d)

function cleanup {
    # Unset the trap to prevent an infinite loop
    trap - SIGINT SIGTERM ERR EXIT

    step rm -rf "$temp_dir"
}

files="{cargo-marker,marker_rustc_driver}-$host_triple.{tar.gz,sha256}"

# Download all files using a single TCP connection with HTTP2 multiplexing
# Unfortunately, `curl 7.68.0` installed on Ubuntu 20.04 Github Actions runner
# doesn't have the `--output-dir` option (it was added in `7.73.0`), so we have
# to do an explicit `pushd/popd` for the temp directory.
step pushd "$temp_dir"
step curl \
    --location \
    --silent \
    --fail \
    --show-error \
    --retry 5 \
    --retry-connrefused \
    --remote-name \
    "https://github.com/rust-marker/marker/releases/download/v$version/$files"
step popd

function extract_archive {
    local bin="$1"
    local dest="$2"
    local file_stem="$bin-$host_triple"

    # We have to enter and exit from the temp dir because the destination path may be
    # relative, and we don't want that path to be relative to the temp dir.
    #
    # Another thing:
    # `--ignore-missing` because the `sha256` file also includes the checksum for `zip` archive,
    # but we only download the `tar.gz` one.
    #
    # On MacOS there isn't `sha256sum` command, but there is `shasum` which is compatible.
    (step cd $temp_dir && step shasum --ignore-missing --check "$file_stem".sha256)

    step tar --extract --file "$temp_dir/$file_stem.tar.gz" --directory "$dest"
}

extract_archive cargo-marker "${CARGO_HOME-$HOME/.cargo}/bin"

extract_archive marker_rustc_driver "$(rustc "+$toolchain" --print sysroot)/bin"

# We use `+$toolchain` to make sure we don't try to install the default toolchain
# in the workspace via the rustup proxy, but use the toolchain we just installed.
step cargo "+$toolchain" marker --version
