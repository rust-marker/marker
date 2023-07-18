#!/usr/bin/env bash

set -euo pipefail

script_dir=$(readlink -f $(dirname $0))

. $script_dir/lib.sh

# Typos check should be pinned to the very specific version
# to prevent sudden dictionary updates from making our CI fail
version=v1.16.1

base_url="https://github.com/crate-ci/typos/releases/download/$version"

file_stem="typos-$version-x86_64-unknown-linux-musl"

curl_and_decompress $base_url/$file_stem.tar.gz ./typos
