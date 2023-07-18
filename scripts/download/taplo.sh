#!/usr/bin/env bash

set -euo pipefail

script_dir=$(readlink -f $(dirname $0))

. $script_dir/lib.sh

version=0.7.0

base_url=https://github.com/tamasfe/taplo/releases/download/release-taplo-cli-$version

download_and_decompress $base_url/taplo-$arch_rust-unknown-linux-gnu.tar.gz taplo
