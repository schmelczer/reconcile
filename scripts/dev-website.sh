#!/bin/bash

set -e

wasm-pack build --target web --features wasm,wee_alloc
cd reconcile-js
npm run build
cd ../examples/website
npm run start
