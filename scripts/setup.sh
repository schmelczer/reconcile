#!/bin/bash

set -e
cd "$(dirname "$0")/.."

node_version=$(node --version | cut -d'.' -f1 | tr -d 'v')
if [ "$node_version" != "22" ]; then
  echo "Error: Node.js version 22 is required, but found version $node_version"
  exit 1
fi

command -v wasm-pack || cargo install --locked wasm-pack
command -v cargo-machete || cargo install --locked cargo-machete

npm ci --prefix reconcile-js
npm ci --prefix examples/website
cp README.md LICENSE reconcile-js/
cp README.md LICENSE reconcile-python/
uv sync --directory reconcile-python --frozen --no-install-project
