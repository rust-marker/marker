#!/usr/bin/env bash

set -euo pipefail

# Parse the first numbered version from the changelog.
# This lives in a file separate from CI scripts so that it could be tested locally.

grep --perl-regexp --only-matching '## \[\K\d+\.\d+\.\d+(-\w+)?' CHANGELOG.md \
    | head --lines 1
