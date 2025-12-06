#!/bin/bash

set -e

which wasm-pack || cargo install wasm-pack
wasm-pack build --target web --features wasm

cd reconcile-js
npm ci
npm run build
cd ../examples/website
npm ci
npm run build
cd -
