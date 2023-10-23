#!/usr/bin/env bash

set -euo pipefail

script_dir=$(readlink -f $(dirname ${BASH_SOURCE[0]}))

. $script_dir/../lib.sh

# The tag of the release that will be updated with the new artifact
tag="$1"

# Path to the file on the local file system that will be uploaded to GitHub
file="$2"

# The name of the artifact that will be uploaded to GitHub
artifact="$3"

with_log tar --auto-compress --create --file "$artifact.tar.gz" "$file"
with_log zip -r $artifact.zip $file

with_log shasum -a 256 "$artifact.tar.gz" "$artifact.zip" > "$artifact.sha256"

with_backoff gh release upload \
    "$tag" \
    "$artifact.tar.gz" \
    "$artifact.zip" \
    "$artifact.sha256" \
    --clobber
