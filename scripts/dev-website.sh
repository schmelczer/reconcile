#!/bin/bash

set -e

wasm-pack build --target web --features wasm

cd reconcile-js
npm install
npm run build

cd ../examples/website
npm install
npm run start
