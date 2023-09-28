#!/usr/bin/env bash

set -euo pipefail

script_dir=$(readlink -f $(dirname ${BASH_SOURCE[0]}))

. $script_dir/lib.sh

version=0.23.0

base_url=https://github.com/obi1kenobi/cargo-semver-checks/releases/download/v$version

file_stem=cargo-semver-checks-$triple_rust

download_and_decompress $base_url/$file_stem.tar.gz ./cargo-semver-checks

move_to_path cargo-semver-checks
