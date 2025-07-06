#!/bin/bash

set -e

cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged
cargo fmt --all

cd reconcile-js
npm run format

cd ../examples/website
npm run format

echo "Success!"
