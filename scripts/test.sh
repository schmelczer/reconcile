#!/bin/bash

set -e

wasm-pack build --target web --features wasm,console_error_panic_hook
cargo test --verbose --features serde -- --include-ignored 

cargo test 
cargo test --features serde
cargo test --features wasm
cargo test --features all

wasm-pack test --node --features wasm,console_error_panic_hook

cd reconcile-js
npm install
npm run test
cd -

echo "Success!"
