#!/bin/bash

if [[ $1 != nightly-????-??-?? ]]
then
    echo "Please enter a valid toolchain like \`nightly-2022-01-01\`"
    exit 1
fi

sed -i "s/nightly-2023-10-05/$1/g" \
    ./marker_rustc_driver/src/main.rs \
    ./marker_rustc_driver/README.md \
    ./rust-toolchain.toml \
    ./.github/workflows/* \
    ./scripts/update-toolchain.sh \
    ./cargo-marker/src/backend/driver.rs \
    ./cargo-marker/README.md \
    ./scripts/release/install.sh \
    ./scripts/release/install.ps1
