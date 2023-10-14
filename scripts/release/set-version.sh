#!/usr/bin/env bash

set -euo pipefail

script_dir=$(readlink -f $(dirname ${BASH_SOURCE[0]}))

. $script_dir/../lib.sh
. $script_dir/../replace-in-regions.sh

new_version="$1"
shift

commit=""

while [[ $# -gt 0 ]]
do
    case $1 in
        --commit)
            commit="$2"
            shift
            shift
            ;;
        *)
            die "Unknown option: $1"
            ;;
    esac
done

# Set the rust toolchain in the installation scripts and other docs to the version
# that was used in the development of the current release.
rust_toolchain=$(sed -n 's/^channel\s*=\s*"nightly-\(.*\)"/\1/p' rust-toolchain.toml)

replace_date_in_regions "rust toolchain release" "$rust_toolchain"

# Dev means all kinds of versions including stable `x.y.z`, unstable `x.y.z-suffix`
# and dev `x.y.z-dev`. Yes, we treat `-dev` as a special one that we never release.
replace_semver_in_regions "marker version dev" "$new_version"

# Unstable means all kinds of versions including unstable `x.y.z-suffix`, but excluding `x.y.z-dev`
if [[ "$new_version" != *-dev ]]; then
    replace_semver_in_regions "marker version unstable" "$new_version"
fi

# Only suffixless `x.y.z` versions are replaced in stable mode
if [[ "$new_version" != *-* ]]; then
    replace_semver_in_regions "marker version stable" "$new_version"

    # Special case for the GitHub action syntax used in examples of `yml` markdown
    # fenced code blocks where there we can't limit the region to exclude other
    # semver versions of other GitHub actions in the region.
    replace_semver_in_regions \
        "marker action version stable" \
        "$new_version" \
        --prefix "rust-marker\/marker@"
fi

# We need any version of cargo executable to update the `Cargo.lock` file.
# Github runners have `cargo` installed by default, but because this repo
# contains a `rust-toolchain.toml` file the bare `cargo` command will try
# to install the toolchain specified in this file. We don't need that, so
# we use `cargo +stable` to force the use of the stable toolchain that should
# be installed by default.
with_log cargo +stable update --workspace

if [[ $commit != "" ]]; then
    with_log git commit --all --message "$commit"
fi
