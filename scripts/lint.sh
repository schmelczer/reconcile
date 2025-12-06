#!/bin/bash

set -e

which cargo-machete || cargo install cargo-machete
cargo machete

cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged
cargo fmt --all

cd reconcile-js
npm ci
npm run format

cd ../examples/website
npm ci
npm run format

echo "Success!"
