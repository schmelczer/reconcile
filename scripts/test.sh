#!/bin/bash

set -e

wasm-pack build --target web --features wasm
cargo test --verbose --features serde -- --include-ignored 

cargo test 
cargo test --features serde
cargo test --features wasm
cargo test --features all

wasm-pack test --node --features wasm

cd reconcile-js
npm install
npm run test
cd -

echo "Success!"
