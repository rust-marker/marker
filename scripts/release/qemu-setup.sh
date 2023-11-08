#!/usr/bin/env bash
# This script is intended to be run inside of a QEMU VM used for building
# the `marker_rustc_driver` binary for ARM targets. We use a bare ubuntu
# image in this environment which doesn't have anything installed for building
# a Rust project, so we install all the essential build tools here.

set -euo pipefail

function apt-get-install {
    with_backoff apt-get update -y
    with_backoff apt-get install -y --no-install-recommends --no-install-suggests "$@"
}

apt-get-install \
    build-essential \
    ca-certificates \
    curl

function curl_with_retry {
    with_log curl \
        --location \
        --silent \
        --show-error \
        --fail \
        --retry 5 \
        --retry-all-errors \
        "$@"
}

start_group "Installing Rust toolchain $rust_version"
curl_with_retry \
    --proto '=https' \
    --tlsv1.2 \
    -sSf https://sh.rustup.rs \
    | with_log sh -s -- \
        -y \
        --default-toolchain $rust_version \
        --no-modify-path \
        --component rust-src rustc-dev llvm-tools
end_group
