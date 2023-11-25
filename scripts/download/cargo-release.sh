#!/usr/bin/env bash

set -euo pipefail

script_dir=$(readlink -f $(dirname ${BASH_SOURCE[0]}))

. $script_dir/lib.sh

version=0.25.0

base_url=https://github.com/crate-ci/cargo-release/releases/download/v$version

file_stem=cargo-release-v$version-$triple_rust

download_and_decompress $base_url/$file_stem.tar.gz ./cargo-release

move_to_path cargo-release
