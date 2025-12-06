#!/bin/bash

set -e

which cargo-machete || cargo install cargo-machete
cargo machete

cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged
cargo fmt --all

cd reconcile-js
npm run format

cd ../examples/website
npm run format

echo "Success!"
