#!/usr/bin/env bash

set -euo pipefail

script_dir=$(readlink -f $(dirname $0))

. $script_dir/lib.sh

version=0.9.57

download_and_decompress https://get.nexte.st/$version/$arch_rust-unknown-linux-gnu.tar.gz
