#!/usr/bin/env bash

set -euo pipefail

script_dir=$(readlink -f $(dirname ${BASH_SOURCE[0]}))

. $script_dir/lib.sh

version=0.5.0

base_url=https://github.com/bnjbvr/cargo-machete/releases/download/v$version
file_stem=cargo-machete-v$version-$arch_rust-unknown-linux-musl

download_and_decompress \
    --check-hash sha256 \
    $base_url/$file_stem.tar.gz \
    --strip-components 1 $file_stem/cargo-machete
