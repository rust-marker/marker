#!/usr/bin/env bash

set -euo pipefail

version="$1"

IFS='.' read -r major minor patch <<< "$version"

((minor++))

echo "$major.$minor.0-dev"
