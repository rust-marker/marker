#!/usr/bin/env bash
# This script is used to test the release automation script that sets the
# new version number in the project files. It does so by copying the repo
# into a temp directory, staging all dirty files, running the script, and
# comparing the diff to a snapshot of the expected diff.

set -euo pipefail

script_dir=$(readlink -f $(dirname ${BASH_SOURCE[0]}))

. $script_dir/../lib.sh

trap cleanup SIGINT SIGTERM ERR EXIT

temp_dir=$(mktemp -d)

function cleanup {
    # Unset the trap to prevent an infinite loop
    trap - SIGINT SIGTERM ERR EXIT

    with_log rm -rf "$temp_dir"
}

with_log rsync --archive --exclude-from .gitignore . "$temp_dir"

with_log pushd "$temp_dir"

# Make sure any dirty files in the repo at this point don't influence the diff
with_log git add --all

./scripts/release/set-version.sh '0.1.0'

snap=scripts/release/set-version.diff

# sed doesn't support non-capturing groups, so we have to use a capturing one
num='[0-9]+'

# Make the git diff snapshot and sanitize it from the noise and variable parts
#
# There is a caveat here for the major version. It is rather ambiguous, because
# it is just a single number, there are no dots in it that could identify it as
# semver version. So for the major version we require that it is always specified
# with the `v` prefix e.g. `v1`.
actual=$(\
    with_log git diff --unified=1 \
    | grep --invert-match --perl-regexp '^(index)|(@@.*@@ )|(--- .*)|(\+\+\+ .*)' \
    | sed --regexp-extended "
        s/diff --git a\/(.*) b\/.*/\n=== \1 ===/
        /^-/ s/$num\.$num\.$num/X.Y.Z/g
        /^-/ s/$num\.$num/X.Y/g
        /^-/ s/v$num/vX/g
    " \
)

if [[ -v UPDATE_SNAP ]]; then
    with_log popd
    echo "$actual" > "$snap"
    exit 0
fi

err=0

echo "$actual" | git diff --no-index --exit-code "$snap" - || err=1

if [[ $err == 0 ]]; then
    echo "The test snapshot is up to date."
    exit 0
fi

die "$(cat <<EOF
--------------------------------------------------------------------------
The test snapshot is outdated, or the release automation script is broken.
If the change in the snapshot is expected run the following to update it.
UPDATE_SNAP=1 ./scripts/release/test.sh
EOF
)"
