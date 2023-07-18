#!/usr/bin/env bash

set -euo pipefail

script_dir=$(readlink -f $(dirname $0))

. $script_dir/lib.sh

version=0.8.1

base_url="https://github.com/tamasfe/taplo/releases/download/$version"

file_name=taplo-full-linux-x86_64

curl_and_decompress $base_url/$file_name.gz

mv $file_name ./taplo
chmod +x ./taplo
