#!/bin/bash

set -e

rm -rf pkg
wasm-pack build --target web --features wasm,wee_alloc

