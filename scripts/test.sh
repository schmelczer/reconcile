#!/bin/bash

set -e

wasm-pack build --target web --features wasm
cargo test --verbose -- --include-ignored
cargo test --features serde
cargo test --features wasm
wasm-pack test --node --features wasm

cd reconcile-js
npm install
npm run test
cd -

echo "Success!"
