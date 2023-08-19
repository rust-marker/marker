#!/usr/bin/env bash

set -euo pipefail

script_dir=$(readlink -f $(dirname ${BASH_SOURCE[0]}))

. $script_dir/lib.sh

# mdbook
version=v0.4.33
base_url=https://github.com/rust-lang/mdBook/releases/download/$version
file_stem=mdbook-$version-$triple_rust
download_and_decompress $base_url/$file_stem.tar.gz

mv mdbook$exe ~/.cargo/bin

# mdbook-toc
version=0.14.1
base_url=https://github.com/badboy/mdbook-toc/releases/download/$version
file_stem=mdbook-toc-$version-$triple_rust
download_and_decompress $base_url/$file_stem.tar.gz

mv mdbook-toc$exe ~/.cargo/bin
