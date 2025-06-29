#!/bin/bash

set -e

wasm-pack build --target web --features wasm
cargo test --verbose
cargo test --features serde
cargo test --features wasm
wasm-pack test --node --features wasm

echo "Success!"