#!/bin/bash

set -e

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
if ! grep -Fxq "reconcile-text = \"$VERSION\"" README.md; then
  echo "README.md must use the Cargo.toml version ($VERSION) in the Rust installation example."
  exit 1
fi

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
