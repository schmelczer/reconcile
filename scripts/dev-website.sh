#!/bin/bash

set -e

rm -rf pkg

wasm-pack build --target web --features wasm

cp -R pkg/reconcile.js examples/website/
cp -R pkg/reconcile_bg.wasm examples/website/

cd examples/website/

python3 -m http.server $1
