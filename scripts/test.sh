#!/bin/bash

set -e

wasm-pack build --target web --features wasm,wee_alloc
cargo test --verbose
cargo test --features serde
cargo test --features wasm,wee_alloc
wasm-pack test --node --features wasm,wee_alloc

cd reconcile-js
npm run test
cd -

echo "Success!"
