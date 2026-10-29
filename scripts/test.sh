#!/bin/bash

set -e
cd "$(dirname "$0")/.."

cargo test --features serde -- --include-ignored
cargo test
cargo test --features wasm
cargo test --features all
wasm-pack test --node --features wasm,console_error_panic_hook

npm test --prefix reconcile-js
uv run --directory reconcile-python --no-sync pytest -v
