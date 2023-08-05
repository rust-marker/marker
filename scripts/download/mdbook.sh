#!/usr/bin/env bash

set -euo pipefail

script_dir=$(readlink -f $(dirname ${BASH_SOURCE[0]}))

. $script_dir/lib.sh

version=v0.4.32

base_url=https://github.com/rust-lang/mdBook/releases/download/$version

file_stem=mdbook-$version-x86_64-unknown-linux-gnu

download_and_decompress $base_url/$file_stem.tar.gz
