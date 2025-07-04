#!/bin/bash

set -e

wasm-pack build --target web --features wasm
cd reconcile-js
npm run build
cp -R dist ../examples/website

cd ../examples/website

python3 -m http.server $1 --bind 0.0.0.0
