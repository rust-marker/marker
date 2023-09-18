#!/usr/bin/env bash
# This script downloads the `cargo-marker` and the `marker_rustc_driver`
# binaries from the GitHub release assets. It also sets up the required
# Rust toolchain that the `marker_rustc_driver` depends on.
#
# This script must be self-contained! Nothing here should use anything from
# the marker repository, because users are expected to run this script on
# their machines where they don't have the marker repository cloned.

set -Eeuo pipefail

# This script isn't meant to be run from `master`, but if it is, then
# it will install the latest version be it a stable version or a pre-release.
# region replace-version unstable
version=0.2.1
# endregion replace-version unstable

toolchain=nightly-2023-08-24

function with_log {
    echo -e "\033[32;1m❱\033[0m $@" >&2
    "$@"
}

with_log rustup install --profile minimal --no-self-update $toolchain

host_triple=$(\
    rustc +$toolchain --version --verbose \
    | grep --only-matching --perl-regexp 'host: \K.*' \
)

trap cleanup SIGINT SIGTERM ERR EXIT

temp_dir=$(mktemp -d)

function cleanup {
    # Unset the trap to prevent an infinite loop
    trap - SIGINT SIGTERM ERR EXIT

    with_log rm -rf "$temp_dir"
}

files="{cargo-marker,marker_rustc_driver}-$host_triple.{tar.gz,sha256}"

# Download all files using a single TCP connection with HTTP2 multiplexing
with_log curl \
    --location \
    --silent \
    --fail \
    --show-error \
    --retry 5 \
    --retry-connrefused \
    --remote-name \
    --output-dir "$temp_dir" \
    "https://github.com/rust-marker/marker/releases/download/v$version/$files"

function extract_archive {
    local bin="$1"
    local dest="$2"
    local file_stem="$bin-$host_triple"

    # `--ignore-missing` because the `sha256` file also includes the checksum for `zip` archive,
    # but we only download the `tar.gz` one.
    (with_log cd $temp_dir && with_log sha256sum --ignore-missing --check "$file_stem".sha256)

    with_log tar --extract --file "$temp_dir/$file_stem.tar.gz" --directory "$dest"
}

extract_archive cargo-marker "${CARGO_HOME-$HOME/.cargo/bin}"

extract_archive marker_rustc_driver "$(rustc +$toolchain --print sysroot)/bin"
