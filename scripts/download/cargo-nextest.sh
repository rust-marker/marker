#!/usr/bin/env bash

set -euo pipefail

script_dir=$(readlink -f $(dirname $0))

. $script_dir/lib.sh

version=0.9.57

if [[ $os == macos ]]; then
    triple_rust=universal-apple-darwin
fi

download_and_decompress https://get.nexte.st/$version/$triple_rust.tar.gz
