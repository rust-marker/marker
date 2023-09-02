#!/usr/bin/env bash

set -euo pipefail

script_dir=$(readlink -f $(dirname ${BASH_SOURCE[0]}))

. $script_dir/../lib.sh

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

function replace {
    local file="$1"
    local region="replace-version"

    if (( $# == 2 )); then
        region="$region $2"
    fi

    comment_begin="(#|\/\/|<!--)"
    comment_end="( -->)?$"

    with_log sed --regexp-extended --in-place --file - "$file" <<EOF
        /$comment_begin region $region$comment_end/,/$comment_begin endregion $region$comment_end/ \
        s/[0-9]+\.[0-9]+\.[0-9]+(-[0-9a-zA-Z.]+)?/$new_version/
EOF
}

files=($(\
    find . -type f \
        \( -name "*.rs" \
        -o -name "*.md" \
        -o -name "*.toml" \
        -o -name "*.sh" \
        \)\
))

for file in "${files[@]}"
do
    replace "$file" unstable
    if [[ "$new_version" != *-* ]]; then
        replace "$file" stable
    fi
done

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
