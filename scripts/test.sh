#!/bin/bash

set -e

which cargo-insta || cargo install cargo-insta
which wasm-pack || cargo install wasm-pack

node_version=$(node --version | cut -d'.' -f1 | tr -d 'v')
if [ "$node_version" != "22" ]; then
    echo "Error: Node.js version 22 is required, but found version $node_version"
    exit 1
fi

wasm-pack build --target web --features wasm,console_error_panic_hook
cargo test --verbose --features serde -- --include-ignored 

cargo test 
cargo test --features serde
cargo test --features wasm
cargo test --features all

wasm-pack test --node --features wasm,console_error_panic_hook

cd reconcile-js
npm ci
npm run build
npm run test
cd -

echo "Success!"
