#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

# if [ -z $CI_PROJECT_NAME ] ; then
#    rustup update nightly
#    rustup update nightly-2020-10-05
#    rustup update stable
# fi
#
# rustup target add wasm32-unknown-unknown --toolchain nightly

# 1.49 does not compile substrate 2.0.0
rustup uninstall stable
rustup install 1.48.0
rustup default 1.48.0-x86_64-unknown-linux-gnu
rustup target add wasm32-unknown-unknown --toolchain nightly-2020-10-05
