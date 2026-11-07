#!/bin/bash

set -e
cd "$(dirname "$0")/.."

scripts/setup-js-toolchain.sh
command -v cargo-machete || cargo install --locked cargo-machete

npm ci --prefix reconcile-js
npm ci --prefix examples/website
cp README.md LICENSE reconcile-js/
cp README.md LICENSE reconcile-python/
uv sync --directory reconcile-python --frozen --no-install-project
