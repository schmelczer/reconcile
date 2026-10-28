#!/bin/bash

set -e

which cargo-machete || cargo install cargo-machete
cargo machete

cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check

cd reconcile-js
npm ci
npm run format:check

cd ../examples/website
npm ci
npm run format:check

cd ../../reconcile-python
cp ../README.md ../LICENSE .
uv run maturin develop -q
uv run ruff check python/ tests/
uv run ruff format --check python/ tests/
uv run pyright python/ tests/
cd -

echo "Success!"
