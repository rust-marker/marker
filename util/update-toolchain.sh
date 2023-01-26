#!/bin/bash

if [[ $1 == nightly-????-??-?? ]]
then
    sed -i "s/nightly-2023-01-26/$1/g" ./marker_driver_rustc/src/main.rs ./rust-toolchain .github/workflows/* ./util/update-toolchain.sh cargo-marker/src/driver.rs
else
    echo "Please enter a valid toolchain like \`nightly-2022-01-01\`"
fi;
