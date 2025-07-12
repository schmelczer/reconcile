#!/bin/bash

set -e

wasm-pack build --target web --features wasm
cd reconcile-js
npm run build
cd ../examples/website
npm run start
