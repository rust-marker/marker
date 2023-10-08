#!/usr/bin/env bash

set -euo pipefail

. $(dirname ${BASH_SOURCE[0]})/replace-in-regions.sh

replace_date_in_regions "rust toolchain dev" "$1"
