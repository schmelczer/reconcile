#!/bin/bash

set -e

wasm-pack build --target web --features wasm
cargo test --verbose --features serde -- --include-ignored 
cargo test --features serde,wasm
wasm-pack test --node --features wasm

cd reconcile-js
npm install
npm run test
cd -

echo "Success!"
