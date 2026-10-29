#!/bin/bash

set -e
cd "$(dirname "$0")/.."

wasm-pack build --target web --features wasm,console_error_panic_hook
npm run build --prefix reconcile-js
