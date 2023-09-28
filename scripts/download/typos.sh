#!/usr/bin/env bash

set -euo pipefail

script_dir=$(readlink -f $(dirname ${BASH_SOURCE[0]}))

. $script_dir/lib.sh

# Typos check should be pinned to the very specific version
# to prevent sudden dictionary updates from making our CI fail
version=v1.16.1

base_url=https://github.com/crate-ci/typos/releases/download/$version

if [[ $os == linux ]]; then
    triple_rust=$arch_rust-unknown-linux-musl
fi

file_stem=typos-$version-x86_64-unknown-linux-musl

download_and_decompress $base_url/$file_stem.tar.gz ./typos

move_to_path typos
