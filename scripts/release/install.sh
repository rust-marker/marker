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

echo "Bash version: $BASH_VERSION" >&2

# Configure the retries for downloading the release assets
: "${MARKER_NET_RETRY_COUNT:=5}"
: "${MARKER_NET_RETRY_MAX_DELAY:=60}"
: "${RUSTUP_MAX_RETRIES:=$MARKER_NET_RETRY_COUNT}"

# This script isn't meant to be run from `master`, but if it is, then
# it will install the latest version be it a stable version or a pre-release.
# region replace marker version unstable
version=0.4.3-rc
# endregion replace marker version unstable

# region replace rust toolchain release
toolchain=nightly-2023-11-16
# endregion replace rust toolchain release

function step {
    local cmd="$1"
    shift
    echo -e "\033[32;1m❱ \033[1;33m$cmd\033[0m $@" >&2
    $cmd "$@"
}

cat <<EOF
Using config env vars (override these if needed):
    MARKER_NET_RETRY_COUNT=$MARKER_NET_RETRY_COUNT
    MARKER_NET_RETRY_MAX_DELAY=$MARKER_NET_RETRY_MAX_DELAY
    RUSTUP_MAX_RETRIES=$RUSTUP_MAX_RETRIES
EOF

step curl --version
step grep --version
step tar --version

curl_version=$(curl --version | grep -o 'curl [0-9]\+\.[0-9]\+\.[0-9]\+')
curl_major=$(echo "$curl_version" | grep -o ' [0-9]\+\.'  | grep -o '[0-9]\+')
curl_minor=$(echo "$curl_version" | grep -o '\.[0-9]\+\.' | grep -o '[0-9]\+')

echo "Parsed curl major.minor: $curl_major.$curl_minor"

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

# We don't need such a condition in windows installation script because the
# version of curl on windows 2019 is quite recent (8.2.1). However the GitHub
# OS image for ubuntu-20.04 uses curl 7.68.0, and this is the environment where
# this if condition is needed.
if [[ "$curl_major" -lt 7 || $curl_major -eq 7 && "$curl_minor" -lt 71 ]]; then
    echo -e "
\033[33;1m[WARN] Installed curl version is $curl_major.$curl_minor, but \
--retry-all-errors option is supported only since curl 7.71, so this option \
will not be set. This means that if the download fails due to an error HTTP status \
code, it won't be retried. The script will retry only 'connection refused' errors.\033[0m
" >&2
    retry_all_errors="--retry-connrefused"
else
    retry_all_errors="--retry-all-errors"
fi

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
    --retry "$MARKER_NET_RETRY_COUNT" \
    --retry-max-time "$MARKER_NET_RETRY_MAX_DELAY" \
    $retry_all_errors \
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
