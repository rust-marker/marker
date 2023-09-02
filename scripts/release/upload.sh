#!/usr/bin/env bash

set -Eeuo pipefail

script_dir=$(readlink -f $(dirname ${BASH_SOURCE[0]}))

. $script_dir/../lib.sh

tag="$1"
file="$2"

with_log tar --auto-compress --create --file "$file.tar.gz" "$file"
with_log zip -r $file.zip $file

with_log shasum -a 256 "$file.tar.gz" "$file.zip" > "$file.sha256"

with_backoff gh release upload \
    "$tag" \
    "$file.tar.gz" \
    "$file.zip" \
    "$file.sha256" \
    --clobber
