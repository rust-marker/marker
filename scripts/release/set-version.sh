#!/usr/bin/env bash

set -euo pipefail

script_dir=$(readlink -f $(dirname ${BASH_SOURCE[0]}))

. $script_dir/../lib.sh

new_version="$1"

with_log sed --in-place "s/^\(version\s*=\s*\)\".*\"/\1\"$new_version\"/" Cargo.toml

with_log cargo update --workspace
